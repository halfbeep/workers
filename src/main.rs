use tokio::sync::mpsc;  // Tokio's MPSC channel
use std::sync::Arc;
use std::time::Duration;
use rand::Rng;
use tokio::task;

#[derive(Debug)]
enum TaskError {
    ProcessingError(String),
}

#[derive(Debug, Clone)]  // Deriving Clone and Debug
struct Task<T> {
    id: usize,
    data: T,
}

#[derive(Debug, Clone)]  // Deriving Clone and Debug
struct Result<T> {
    id: usize,
    output: T,
    success: bool,
}

async fn process_task<T>(task: Task<T>) -> std::result::Result<Result<T>, TaskError>
where
    T: std::fmt::Debug + Send + Sync + 'static,
{
    let mut rng = rand::thread_rng();
    let processing_time = rng.gen_range(1..5);
    tokio::time::sleep(Duration::from_secs(processing_time)).await;

    if rng.gen_bool(0.2) {
        Err(TaskError::ProcessingError(format!(
            "Task {} failed to process",
            task.id
        )))
    } else {
        Ok(Result {
            id: task.id,
            output: task.data,
            success: true,
        })
    }
}

async fn worker<T>(
    worker_id: usize,
    task_receiver: Arc<tokio::sync::Mutex<mpsc::Receiver<Task<T>>>>,  // Single shared receiver for all workers
    result_sender: mpsc::Sender<Result<T>>,
) where
    T: std::fmt::Debug + Send + Sync + Clone + 'static,  // Ensure T is Send + Sync
{
    println!("Worker {} starting...", worker_id);

    loop {
        // Lock the mutex, receive a task, and then unlock the mutex
        let task = {
            let mut lock = task_receiver.lock().await;
            lock.recv().await
        };

        match task {
            Some(task) => {
                println!("Worker {} processing task {:?}", worker_id, task);

                match process_task(task).await {
                    Ok(result) => {
                        result_sender.send(result.clone()).await.unwrap();
                        println!("Worker {} finished processing task {}", worker_id, result.id);
                    }
                    Err(e) => {
                        println!("Worker {} encountered error: {:?}", worker_id, e);
                    }
                }
            }
            None => {
                println!("Worker {} shutting down", worker_id);
                break;
            }
        }
    }
}

async fn task_scheduler<T>(
    num_workers: usize,
    tasks: Vec<Task<T>>,
) -> std::result::Result<Vec<Result<T>>, TaskError>
where
    T: std::fmt::Debug + Send + Sync + Clone + 'static,
{
    let (task_sender, task_receiver) = mpsc::channel(100);  // Create channel
    let (result_sender, mut result_receiver) = mpsc::channel(100);  // Create result channel

    // Wrap the receiver in an Arc and Mutex for shared access between workers
    let task_receiver = Arc::new(tokio::sync::Mutex::new(task_receiver));

    // Spawn workers
    let mut workers = Vec::new();
    for i in 0..num_workers {
        let task_receiver = Arc::clone(&task_receiver);  // Clone the Arc, not the receiver
        let result_sender = result_sender.clone();

        let worker_handle = tokio::spawn(worker(i, task_receiver, result_sender));  // Spawning tasks
        workers.push(worker_handle);
    }

    // Send tasks to the workers
    for task in tasks {
        task_sender.send(task).await.unwrap();
    }

    // Close the task sender to notify workers that no more tasks are coming
    drop(task_sender);

    // Collect results
    let mut results = Vec::new();
    while let Some(result) = result_receiver.recv().await {
        results.push(result);
    }

    // Wait for all workers to finish
    for worker in workers {
        worker.await.unwrap();
    }

    Ok(results)
}

#[tokio::main]
async fn main() {
    let tasks: Vec<Task<String>> = (0..10)
        .map(|i| Task {
            id: i,
            data: format!("Task data {}", i),
        })
        .collect();

    println!("Starting task scheduler with 4 workers...");

    match task_scheduler(4, tasks).await {
        Ok(results) => {
            println!("All tasks completed. Results:");
            for result in results {
                println!("Task {}: {:?}", result.id, result);
            }
        }
        Err(e) => {
            println!("Task scheduler encountered an error: {:?}", e);
        }
    }
}