;; ABOUTME: Concurrency utilities - Parallel processing and concurrent execution helpers
;; This library provides high-level abstractions for concurrent programming

;; ============================================================================
;; Parallel Data Processing
;; ============================================================================

;;; Apply function to each element in parallel, returning new list.
;;;
;;; **Parameters:**
;;; - f: Function to apply to each element
;;; - lst: Input list
;;;
;;; **Returns:** New list with f applied to each element (order preserved)
;;;
;;; **Time Complexity:** O(1) parallel time (assuming infinite cores), O(n) for spawning/collecting
;;;
;;; **Examples:**
;;; - (parallel-map (lambda (x) (* x 2)) '(1 2 3)) => (2 4 6)
;;; - (parallel-map (lambda (url) (http-request url {:method "GET"})) urls) ; Parallel API calls
;;; - (parallel-map read-file file-paths) ; Parallel file reads
;;;
;;; **Notes:**
;;; - Each element processed in separate goroutine
;;; - Results maintain input order
;;; - Ideal for I/O-bound operations (API calls, file I/O, database queries)
;;; - For CPU-bound work, consider overhead of spawning threads
(define (parallel-map f lst)
  (if (empty? lst)
      '()
      (let ((channels (map (lambda (x) (spawn (lambda () (f x)))) lst)))
        (map channel-recv channels))))

;;; Apply function to each element in parallel with error handling.
;;;
;;; **Parameters:**
;;; - f: Function to apply to each element
;;; - lst: Input list
;;;
;;; **Returns:** List of maps with {:ok value} or {:error message}
;;;
;;; **Time Complexity:** O(1) parallel time (assuming infinite cores), O(n) for spawning/collecting
;;;
;;; **Examples:**
;;; - (parallel-map-link (lambda (x) (/ 10 x)) '(1 2 0))
;;;   => ({:ok 10} {:ok 5} {:error "Division by zero"})
;;; - (parallel-map-link (lambda (url) (http-request url {:method "GET"})) urls) ; Robust parallel API calls
;;;
;;; **Notes:**
;;; - Returns maps with :ok or :error keys
;;; - Use map-get to extract results: (map-get result :ok)
;;; - Never crashes on individual failures
;;; - Ideal for unreliable operations (network, external services)
(define (parallel-map-link f lst)
  (if (empty? lst)
      '()
      (let ((channels (map (lambda (x) (spawn-link (lambda () (f x)))) lst)))
        (map channel-recv channels))))

;;; Short alias for parallel-map (Clojure-style).
;;;
;;; **Parameters:**
;;; - f: Function to apply to each element
;;; - lst: Input list
;;;
;;; **Returns:** New list with f applied to each element (order preserved)
;;;
;;; **Examples:**
;;; - (pmap square '(1 2 3 4)) => (1 4 9 16)
;;; - (pmap read-file paths)
;;;
;;; **Notes:** Identical to parallel-map, shorter for interactive use.
(define (pmap f lst)
  (parallel-map f lst))

;;; Execute function on each element in parallel, ignore results.
;;;
;;; **Parameters:**
;;; - f: Function to apply to each element (for side effects)
;;; - lst: Input list
;;;
;;; **Returns:** nil
;;;
;;; **Time Complexity:** O(1) parallel time (assuming infinite cores), O(n) for spawning/waiting
;;;
;;; **Examples:**
;;; - (parallel-for-each println '(1 2 3)) ; Print all concurrently
;;; - (parallel-for-each write-log-entry events) ; Parallel logging
;;;
;;; **Notes:**
;;; - Use when you only care about side effects, not return values
;;; - Waits for all tasks to complete before returning
;;; - Discards all results
(define (parallel-for-each f lst)
  (if (empty? lst)
      nil
      (begin
        (let ((channels (map (lambda (x) (spawn (lambda () (f x)))) lst)))
          (map channel-recv channels))
        nil)))

;; ============================================================================
;; Pipeline and Fan-out Patterns
;; ============================================================================

;;; Fan-out a single value to multiple processing functions in parallel.
;;;
;;; **Parameters:**
;;; - value: Single value to process
;;; - functions: List of functions to apply
;;;
;;; **Returns:** List of results (one per function)
;;;
;;; **Time Complexity:** O(1) parallel time (assuming infinite cores)
;;;
;;; **Examples:**
;;; - (fan-out 10 (list square cube (lambda (x) (* x 10))))
;;;   => (100 1000 100)
;;; - (fan-out user-id (list get-profile get-orders get-preferences))
;;;   ; Fetch multiple user resources in parallel
;;;
;;; **Notes:**
;;; - Opposite of parallel-map (one value, many functions vs many values, one function)
;;; - Useful for fetching multiple derived values from one source
;;; - Results maintain function order
(define (fan-out value functions)
  (parallel-map (lambda (f) (f value)) functions))

;;; Create a parallel processing pipeline with error handling.
;;;
;;; **Parameters:**
;;; - tasks: List of zero-argument functions to execute
;;;
;;; **Returns:** List of maps with {:ok value} or {:error message}
;;;
;;; **Time Complexity:** O(1) parallel time (assuming infinite cores)
;;;
;;; **Examples:**
;;; - (parallel-pipeline
;;;     (list
;;;       (lambda () (http-request "https://api.example.com/users" {:method "GET"}))
;;;       (lambda () (http-request "https://api.example.com/posts" {:method "GET"}))
;;;       (lambda () (read-file "config.json"))))
;;;   ; Execute independent tasks concurrently
;;;
;;; **Notes:**
;;; - All tasks are zero-argument functions
;;; - Errors isolated per task
;;; - Use for independent parallel operations
(define (parallel-pipeline tasks)
  (if (empty? tasks)
      '()
      (let ((channels (map spawn-link tasks)))
        (map channel-recv channels))))

;; Concurrency library loaded
