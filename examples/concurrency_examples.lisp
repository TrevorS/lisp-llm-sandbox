;; ============================================================================
;; Practical Concurrency Examples for LLM Automation
;; ============================================================================
;; This file demonstrates real-world concurrent programming patterns
;; for automating bash, grep, curl, database queries, and data processing.

;; ============================================================================
;; Example 1: Parallel API Scraping with Error Handling
;; ============================================================================

(define (fetch-all-apis urls)
  "Fetch data from multiple APIs concurrently with robust error handling.

  **Use Case:** Gathering data from multiple services (weather, stocks, news)
  **Pattern:** parallel-map-link ensures errors don't crash the workflow"

  (begin
    (println "Fetching from multiple APIs...")

    ;; Fetch all URLs in parallel
    (define results (parallel-map-link
                      (lambda (url) (http-request url {}))
                      urls))

    ;; Process results: extract successful responses, log errors
    (define successful (filter
                         (lambda (r) (map-has? r "ok"))
                         results))

    (define failed (filter
                     (lambda (r) (map-has? r "error"))
                     results))

    (println (string-append "Successful: " (number->string (length successful))))
    (println (string-append "Failed: " (number->string (length failed))))

    ;; Return just the successful bodies
    (map (lambda (r) (map-get (map-get r "ok") :body)) successful)))

;; Example usage:
;; (define urls '(
;;   "https://api.github.com/users/octocat"
;;   "https://api.github.com/repos/rust-lang/rust"
;;   "https://httpbin.org/delay/1"
;; ))
;; (fetch-all-apis urls)

;; ============================================================================
;; Example 2: Parallel File Processing
;; ============================================================================

(define (process-files-parallel paths processor)
  "Process multiple files concurrently with a given processor function.

  **Use Case:** Log analysis, data transformation, batch processing
  **Pattern:** Each file processed in its own goroutine"

  (begin
    (println (string-append "Processing " (number->string (length paths)) " files in parallel..."))

    ;; Read and process each file concurrently
    (define process-one-file
      (lambda (path)
        (begin
          (define content (read-file path))
          (processor path content))))

    ;; Execute with error handling
    (define results (parallel-map-link process-one-file paths))

    ;; Aggregate results
    (define successes (count (lambda (r) (map-has? r "ok")) results))
    (define failures (count (lambda (r) (map-has? r "error")) results))

    (println (string-append "Completed: " (number->string successes) " succeeded, "
                           (number->string failures) " failed"))

    results))

;; Example processor: count lines in each file
(define (count-lines-processor path content)
  "Count lines in a file and return as map"
  (let ((lines (length (string-lines content))))
    {:path path :lines lines}))

;; Example usage:
;; (define files '("log1.txt" "log2.txt" "log3.txt"))
;; (process-files-parallel files count-lines-processor)

;; ============================================================================
;; Example 3: ETL Pipeline - Extract, Transform, Load in Parallel
;; ============================================================================

(define (etl-pipeline sources transformer loader)
  "Extract data from multiple sources, transform in parallel, and load results.

  **Use Case:** Data migration, batch imports, aggregation
  **Pattern:** Fan-out extraction, parallel transform, sequential load"

  (begin
    (println "Starting ETL pipeline...")

    ;; EXTRACT: Fetch from all sources concurrently
    (println "  [Extract] Fetching from sources...")
    (define extract-results (parallel-map-link
                              (lambda (src) (src))  ;; Each source is a zero-arg function
                              sources))

    ;; Filter out errors
    (define extracted-data
      (map (lambda (r) (map-get r "ok"))
           (filter (lambda (r) (map-has? r "ok")) extract-results)))

    (println (string-append "  [Extract] Retrieved " (number->string (length extracted-data)) " datasets"))

    ;; TRANSFORM: Apply transformation to each dataset in parallel
    (println "  [Transform] Processing data...")
    (define transformed-data (parallel-map transformer extracted-data))

    (println (string-append "  [Transform] Processed " (number->string (length transformed-data)) " datasets"))

    ;; LOAD: Apply loader to each transformed dataset
    (println "  [Load] Writing results...")
    (map loader transformed-data)

    (println "ETL pipeline complete!")
    {:extracted (length extracted-data)
     :transformed (length transformed-data)
     :loaded (length transformed-data)}))

;; Example: Aggregate data from multiple JSON endpoints
(define (demo-etl-pipeline)
  "Demo ETL: fetch user data, transform, and save"

  (define sources
    (list
      (lambda () (http-request "https://jsonplaceholder.typicode.com/users/1" {:method "GET"}))
      (lambda () (http-request "https://jsonplaceholder.typicode.com/users/2" {:method "GET"}))
      (lambda () (http-request "https://jsonplaceholder.typicode.com/users/3" {:method "GET"}))))

  (define transformer
    (lambda (data)
      ;; Extract just name and email
      (let ((parsed (json:decode (map-get data :body))))
        {:name (map-get parsed "name")
         :email (map-get parsed "email")})))

  (define loader
    (lambda (record)
      (println (string-append "  Loaded: " (map-get record :name)))))

  (etl-pipeline sources transformer loader))

;; ============================================================================
;; Example 4: Parallel Database Query Pattern
;; ============================================================================

(define (parallel-query-shards query shards)
  "Execute query across multiple database shards concurrently.

  **Use Case:** Distributed database queries, multi-region fetches
  **Pattern:** Query all shards, merge results"

  (begin
    (println (string-append "Querying " (number->string (length shards)) " shards..."))

    ;; Execute query on each shard concurrently
    (define shard-results
      (parallel-map-link
        (lambda (shard)
          ;; In real use, this would be a database connection
          ;; For demo, just simulate with shard info
          (begin
            ;; Simulate query execution time
            {:shard shard :results (list 1 2 3)}))
        shards))

    ;; Merge all successful results
    (define all-results
      (reduce
        (lambda (acc r)
          (if (map-has? r "ok")
              (append acc (map-get (map-get r "ok") :results))
              acc))
        '()
        shard-results))

    (println (string-append "  Retrieved " (number->string (length all-results)) " total records"))
    all-results))

;; Example usage:
;; (parallel-query-shards "SELECT * FROM users" '("shard1" "shard2" "shard3"))

;; ============================================================================
;; Example 5: Batch API Requests with Rate Limiting Pattern
;; ============================================================================

(define (batch-api-calls items api-call batch-size)
  "Process items in batches to avoid overwhelming API or respecting rate limits.

  **Use Case:** Bulk operations with API rate limits
  **Pattern:** Split into batches, process each batch in parallel"

  (begin
    ;; Helper: split list into chunks
    (define (chunk-list lst n)
      (if (empty? lst)
          '()
          (cons (take n lst)
                (chunk-list (drop n lst) n))))

    ;; Split into batches
    (define batches (chunk-list items batch-size))
    (println (string-append "Processing " (number->string (length items))
                           " items in " (number->string (length batches)) " batches"))

    ;; Process each batch sequentially, items within batch in parallel
    (define (process-batch batch)
      (parallel-map-link api-call batch))

    ;; Process all batches (could also parallelize this for more throughput)
    (map process-batch batches)))

;; Example: bulk user lookup
;; (define user-ids '(1 2 3 4 5 6 7 8 9 10))
;; (batch-api-calls user-ids
;;   (lambda (id) (http-request (string-append "https://api.example.com/users/" (number->string id)) {:method "GET"}))
;;   3)

;; ============================================================================
;; Example 6: Fan-out Pattern - Fetch Multiple Resources for One Entity
;; ============================================================================

(define (fetch-user-complete user-id)
  "Fetch all data for a user from multiple services concurrently.

  **Use Case:** Aggregating user profile from microservices
  **Pattern:** Fan-out single ID to multiple endpoints"

  (begin
    (println (string-append "Fetching complete data for user " (number->string user-id)))

    ;; Define fetchers for different aspects
    (define fetchers
      (list
        ;; Profile
        (lambda (id)
          {:service "profile"
           :data {:name "Alice" :age 30}})
        ;; Orders
        (lambda (id)
          {:service "orders"
           :data (list "order1" "order2")})
        ;; Preferences
        (lambda (id)
          {:service "preferences"
           :data {:theme "dark" :lang "en"}})))

    ;; Fan out to all services in parallel
    (define results (fan-out user-id fetchers))

    ;; Combine into single user object
    (reduce
      (lambda (acc result)
        (map-set acc (map-get result :service) (map-get result :data)))
      {}
      results)))

;; Example usage:
;; (fetch-user-complete 42)

;; ============================================================================
;; Example 7: Parallel Pipeline - Independent Task Execution
;; ============================================================================

(define (data-collection-pipeline)
  "Execute multiple independent data collection tasks concurrently.

  **Use Case:** Dashboard data aggregation, status checks
  **Pattern:** All tasks start simultaneously, results aggregated"

  (begin
    (println "Starting parallel data collection...")

    (define tasks
      (list
        ;; Task 1: Get system stats
        (lambda ()
          {:task "system-stats"
           :cpu 45.2
           :memory 78.5})

        ;; Task 2: Count active users
        (lambda ()
          {:task "active-users"
           :count 1234})

        ;; Task 3: Latest errors
        (lambda ()
          {:task "error-count"
           :errors 3})))

    (define results (parallel-pipeline tasks))

    ;; Extract successful results
    (map (lambda (r) (map-get r "ok"))
         (filter (lambda (r) (map-has? r "ok")) results))))

;; Example usage:
;; (data-collection-pipeline)

;; ============================================================================
;; Best Practices Summary
;; ============================================================================

;; 1. **Use parallel-map-link for unreliable operations**
;;    - API calls, network requests, external services
;;    - Returns {:ok value} or {:error message}
;;    - Never crashes, always returns

;; 2. **Use parallel-map for pure computations**
;;    - Data transformations, calculations
;;    - Faster (no error wrapping overhead)
;;    - Assumes operations won't fail

;; 3. **Use fan-out for single entity, multiple operations**
;;    - User profile aggregation
;;    - Multi-service lookups
;;    - Related data fetching

;; 4. **Use parallel-pipeline for independent tasks**
;;    - Dashboard data collection
;;    - Health checks
;;    - Batch jobs

;; 5. **Batch large datasets to control concurrency**
;;    - Prevents overwhelming servers
;;    - Respects rate limits
;;    - Controls memory usage

;; ============================================================================
;; Production Tips
;; ============================================================================

;; - Always use error handling (spawn-link, parallel-map-link)
;; - Consider batching for large datasets (10K+ items)
;; - Monitor for goroutine leaks (ensure channels are consumed)
;; - Use timeouts for long-running operations
;; - Log errors for debugging (don't silently swallow)
;; - Test with small datasets first
;; - Profile to ensure parallelism helps (I/O-bound wins, CPU-bound may not)
