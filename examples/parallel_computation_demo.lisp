;; ============================================================================
;; Parallel Computation Demo - Runnable Examples
;; ============================================================================
;; This file contains working examples you can run immediately

;; Helper function to convert any value to string representation
(define (to-string val)
  "Convert value to string for display"
  (if (number? val)
      (number->string val)
      ;; For lists and other types, just use a placeholder
      "(list)"))

(println "")
(println "==============================================")
(println "   PARALLEL COMPUTATION DEMONSTRATIONS")
(println "==============================================")
(println "")

;; ============================================================================
;; Demo 1: Speed Comparison - Sequential vs Parallel
;; ============================================================================

(println "Demo 1: Sequential vs Parallel Processing")
(println "-------------------------------------------")

;; Simulate expensive computation
(define (expensive-compute x)
  (begin
    ;; Compute something non-trivial
    (define result (* x x x))
    (define result2 (+ result x))
    (define result3 (* result2 result2))
    result3))

;; Sequential processing
(define data '(1 2 3 4 5 6 7 8))
(println "  Processing 8 items...")

(println "  Sequential map:")
(define seq-result (map expensive-compute data))
(println (string-append "    Result: " (number->string (length seq-result)) " items processed"))

(println "  Parallel map:")
(define par-result (parallel-map expensive-compute data))
(println (string-append "    Result: " (number->string (length par-result)) " items processed"))

(println "  Results match: " (= seq-result par-result))
(println "")

;; ============================================================================
;; Demo 2: Error Handling with parallel-map-link
;; ============================================================================

(println "Demo 2: Robust Error Handling")
(println "-------------------------------")

;; Function that fails on zero
(define (divide-100-by x)
  (/ 100 x))

(define test-values '(10 5 0 2 1))
(println (string-append "  Dividing 100 by each of: " (to-string test-values)))

(define results (parallel-map-link divide-100-by test-values))
(println "  Results:")

;; Helper to display result
(define (show-result idx r)
  (begin
    (print (string-append "    [" (number->string idx) "] "))
    (if (map-has? r :ok)
        (println (string-append "Success: " (number->string (map-get r :ok))))
        (println (string-append "Error: " (map-get r :error))))))

;; Display each result
(define (show-results-helper lst idx)
  (if (empty? lst)
      nil
      (begin
        (show-result idx (car lst))
        (show-results-helper (cdr lst) (+ idx 1)))))

(show-results-helper results 1)

(println "  Notice: Error on index 3 (division by zero) didn't crash the program!")
(println "")

;; ============================================================================
;; Demo 3: Fan-out Pattern - One Input, Many Operations
;; ============================================================================

(println "Demo 3: Fan-out Pattern")
(println "------------------------")

(define input-number 7)
(println (string-append "  Input: " (number->string input-number)))

(define operations
  (list
    (lambda (x) (* x x))           ;; Square
    (lambda (x) (* x x x))         ;; Cube
    (lambda (x) (+ x 100))         ;; Add 100
    (lambda (x) (* x 2))           ;; Double
    (lambda (x) (- x 5))))         ;; Subtract 5

(println "  Applying 5 operations in parallel:")
(define fan-results (fan-out input-number operations))
(println (string-append "    Square:      " (number->string (nth 0 fan-results))))
(println (string-append "    Cube:        " (number->string (nth 1 fan-results))))
(println (string-append "    Add 100:     " (number->string (nth 2 fan-results))))
(println (string-append "    Double:      " (number->string (nth 3 fan-results))))
(println (string-append "    Subtract 5:  " (number->string (nth 4 fan-results))))
(println "")

;; ============================================================================
;; Demo 4: Parallel Pipeline - Independent Tasks
;; ============================================================================

(println "Demo 4: Parallel Pipeline (Independent Tasks)")
(println "-----------------------------------------------")

(define tasks
  (list
    (lambda () (begin
                 (println "    Task 1: Computing sum of 1..100")
                 (sum (range 1 101))))
    (lambda () (begin
                 (println "    Task 2: Computing product of 1..10")
                 (product (range 1 11))))
    (lambda () (begin
                 (println "    Task 3: Counting even numbers in 1..50")
                 (count even? (range 1 51))))))

(println "  Launching 3 independent tasks in parallel:")
(define task-results (parallel-pipeline tasks))

(println "")
(println "  Results:")
(println (string-append "    Task 1 (sum 1..100):      {:ok "
                       (number->string (map-get (nth 0 task-results) :ok)) "}"))
(println (string-append "    Task 2 (product 1..10):   {:ok "
                       (number->string (map-get (nth 1 task-results) :ok)) "}"))
(println (string-append "    Task 3 (count evens):     {:ok "
                       (number->string (map-get (nth 2 task-results) :ok)) "}"))
(println "")

;; ============================================================================
;; Demo 5: Data Transformation Pipeline
;; ============================================================================

(println "Demo 5: Parallel Data Transformation")
(println "--------------------------------------")

(define raw-data '(
  {:name "alice" :score 85}
  {:name "bob" :score 92}
  {:name "charlie" :score 78}
  {:name "diana" :score 95}
  {:name "eve" :score 88}))

(println "  Input: 5 student records")
(println "  Transforming each record in parallel...")

;; Transformation: capitalize name, add grade
(define (transform-record record)
  (let ((name (map-get record :name))
        (score (map-get record :score)))
    {:name (string-capitalize name)
     :score score
     :grade (if (>= score 90) "A"
               (if (>= score 80) "B"
                  (if (>= score 70) "C" "F")))}))

(define transformed (parallel-map transform-record raw-data))

(println "  Results:")
(define (show-student s)
  (println (string-append "    "
                         (map-get s :name) " - "
                         "Score: " (number->string (map-get s :score)) ", "
                         "Grade: " (map-get s :grade))))

(map show-student transformed)
(println "")

;; ============================================================================
;; Demo 6: Simulated Batch Processing
;; ============================================================================

(println "Demo 6: Parallel Batch Processing")
(println "-----------------------------------")

;; Simulate processing files
(define files '("data1.txt" "data2.txt" "data3.txt" "data4.txt" "data5.txt"))

(define (process-file filename)
  {:file filename :lines (+ 10 (% (* (string-length filename) 7) 20))})

(println (string-append "  Processing " (number->string (length files)) " files in parallel..."))
(define file-results (parallel-map-link process-file files))

(println "  Results:")
(define successes (filter (lambda (r) (map-has? r :ok)) file-results))
(define failures (filter (lambda (r) (map-has? r :error)) file-results))

(println (string-append "    Successful: " (number->string (length successes))))
(println (string-append "    Failed:     " (number->string (length failures))))

(println "  Successful files:")
(map (lambda (r)
       (let ((data (map-get r :ok)))
         (println (string-append "    - " (map-get data :file)
                                ": " (number->string (map-get data :lines)) " lines"))))
     successes)
(println "")

;; ============================================================================
;; Demo 7: Nested Parallel Operations
;; ============================================================================

(println "Demo 7: Nested Parallelism")
(println "---------------------------")

;; Process groups of data in parallel, each group processed in parallel
(define groups '(
  (1 2 3)
  (4 5 6)
  (7 8 9)))

(println "  Processing 3 groups, each group in parallel:")

(define (process-group group)
  (begin
    (println (string-append "    Processing group: " (to-string group)))
    (define group-results (parallel-map square group))
    (sum group-results)))

(define group-sums (parallel-map process-group groups))

(println "")
(println "  Group sums:")
(println (string-append "    Group 1: " (number->string (nth 0 group-sums))))
(println (string-append "    Group 2: " (number->string (nth 1 group-sums))))
(println (string-append "    Group 3: " (number->string (nth 2 group-sums))))
(println (string-append "    Total:   " (number->string (sum group-sums))))
(println "")

;; ============================================================================
;; Summary
;; ============================================================================

(println "==============================================")
(println "   SUMMARY")
(println "==============================================")
(println "All demos completed successfully!")
(println "")
(println "Key Patterns Demonstrated:")
(println "  1. parallel-map       - Fast parallel data transformation")
(println "  2. parallel-map-link  - Robust error handling")
(println "  3. fan-out            - One input, many operations")
(println "  4. parallel-pipeline  - Independent task execution")
(println "  5. Batch processing   - Handling partial failures")
(println "  6. Nested parallelism - Parallel within parallel")
(println "")
(println "These patterns enable efficient automation of:")
(println "  - API calls and data fetching")
(println "  - File processing and I/O")
(println "  - Data transformations")
(println "  - ETL pipelines")
(println "  - Batch operations")
(println "==============================================")
