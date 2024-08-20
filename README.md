workers

output:-

Starting task scheduler with 4 workers...
Worker 0 starting...
Worker 1 starting...
Worker 2 starting...
Worker 3 starting...
Worker 0 processing task Task { id: 0, data: "Task data 0" }
Worker 1 processing task Task { id: 1, data: "Task data 1" }
Worker 3 processing task Task { id: 3, data: "Task data 3" }
Worker 2 processing task Task { id: 2, data: "Task data 2" }
Worker 0 finished processing task 0
Worker 1 finished processing task 1
Worker 3 encountered error: ProcessingError("Task 3 failed to process")

All tasks completed. Results:
Task 0: Result { id: 0, output: "Task data 0", success: true }
Task 1: Result { id: 1, output: "Task data 1", success: true }
Task 2: Result { id: 2, output: "Task data 2", success: true }
