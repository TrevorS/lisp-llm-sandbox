;; Test script to demonstrate standard library functions

(println "=== Standard Library Demo ===")
(println "")

;; Map example
(println "Map: double each element")
(define doubled (map (lambda (x) (* x 2)) '(1 2 3 4 5)))
(print "Result: ")
(println doubled)
(println "")

;; Filter example
(println "Filter: keep only numbers > 2")
(define filtered (filter (lambda (x) (> x 2)) '(1 2 3 4 5)))
(print "Result: ")
(println filtered)
(println "")

;; Reduce example
(println "Reduce: sum all numbers")
(define total (reduce + 0 '(1 2 3 4 5)))
(print "Result: ")
(println total)
(println "")

;; Reverse example
(println "Reverse: reverse a list")
(define reversed (reverse '(1 2 3 4 5)))
(print "Result: ")
(println reversed)
(println "")

;; Range example
(println "Range: create list from 0 to 9")
(define nums (range 0 10))
(print "Result: ")
(println nums)
(println "")

;; Factorial example
(println "Factorial: compute 5!")
(define fact5 (factorial 5))
(print "Result: ")
(println fact5)
(println "")

;; Compose example
(println "Compose: combine double and increment")
(define double (lambda (x) (* x 2)))
(define inc (lambda (x) (+ x 1)))
(define double-then-inc (compose inc double))
(print "Result of (compose inc double)(5): ")
(println (double-then-inc 5))
(println "")

;; All/Any predicates
(println "All: check if all numbers > 0")
(define all-positive (all (lambda (x) (> x 0)) '(1 2 3 4 5)))
(print "Result: ")
(println all-positive)
(println "")

(println "Any: check if any number > 4")
(define has-large (any (lambda (x) (> x 4)) '(1 2 3 4 5)))
(print "Result: ")
(println has-large)
(println "")

;; Sum and product
(println "Sum: sum of 1..5")
(print "Result: ")
(println (sum '(1 2 3 4 5)))
(println "")

(println "Product: product of 1..5")
(print "Result: ")
(println (product '(1 2 3 4 5)))
(println "")

;; ============================================================================
;; String Functions (string.lisp)
;; ============================================================================

(println "=== String Functions ===")
(println "")

(println "String-capitalize: capitalize first character")
(print "Result: ")
(println (string-capitalize "hello world"))
(println "")

(println "String-concat: join list of strings")
(print "Result: ")
(println (string-concat '("Hello" " " "World" "!")))
(println "")

(println "String-reverse: reverse a string")
(print "Result: ")
(println (string-reverse "Hello"))
(println "")

(println "String-repeat: repeat string n times")
(print "Result: ")
(println (string-repeat "Ha" 3))
(println "")

(println "String-words: split by whitespace")
(print "Result: ")
(println (string-words "Hello world from Lisp"))
(println "")

(println "String-lines: split by newlines")
(print "Result: ")
(println (string-lines "Line 1\nLine 2\nLine 3"))
(println "")

(println "String-pad-left: pad string to width")
(print "Result: ")
(println (string-pad-left "42" 5 "0"))
(println "")

;; ============================================================================
;; Test Framework (test.lisp)
;; ============================================================================

(println "=== Test Framework ===")
(println "")

(println "Define-test: register tests and run them")

;; Register a few tests
(define-test "addition" (assert-equal (+ 2 2) 4))
(define-test "multiplication" (assert-equal (* 3 4) 12))
(define-test "list-reverse" (assert-equal (reverse '(1 2 3)) '(3 2 1)))

;; Run all tests and print results
(define test-results (run-all-tests))
(print-test-summary test-results)

;; Clear tests for clean slate
(clear-tests)
(println "")

;; ============================================================================
;; HTTP Functions (http.lisp)
;; ============================================================================

(println "=== HTTP Functions (commented - requires network) ===")
(println "")

;; NOTE: These examples are commented out because they require network access
;; and an external API. Uncomment to test if network is enabled.

;; (println "HTTP request example:")
;; (define response (http-request "https://jsonplaceholder.typicode.com/users/1" {:method "GET"}))
;; (println "Status: " (http:status response))
;; (println "Success: " (http:check-status response))
;; (println "Body preview: " (substring (http:body response) 0 50))
;; (println "")

(println "HTTP functions available:")
(println "  - http:check-status: check if response is 2xx")
(println "  - http:body: extract response body")
(println "  - http:status: extract status code")
(println "")

;; ============================================================================
;; Concurrency Functions (concurrency.lisp)
;; ============================================================================

(println "=== Concurrency Functions (commented - uses spawn) ===")
(println "")

;; NOTE: These examples are commented out because they use spawn/channels
;; which may not work in all execution environments. Uncomment to test.

;; (println "Parallel-map: map function in parallel")
;; (define parallel-doubled (parallel-map (lambda (x) (* x 2)) '(1 2 3 4 5)))
;; (print "Result: ")
;; (println parallel-doubled)
;; (println "")

;; (println "Fan-out: execute function on multiple inputs in parallel")
;; (define square-fn (lambda (x) (* x x)))
;; (define parallel-squares (fan-out square-fn '(1 2 3 4 5)))
;; (print "Result: ")
;; (println parallel-squares)
;; (println "")

(println "Concurrency functions available:")
(println "  - parallel-map: map in parallel using spawn")
(println "  - parallel-map-link: map with error propagation")
(println "  - pmap: alias for parallel-map")
(println "  - parallel-for-each: parallel side effects")
(println "  - fan-out: parallel function execution")
(println "  - parallel-pipeline: pipeline stages in parallel")
(println "")

;; ============================================================================
;; Map Utilities (core.lisp)
;; ============================================================================

(println "=== Map Utilities ===")
(println "")

(println "Map:query: get value with default")
(define user-map {:name "Alice" :age 30})
(print "Name: ")
(println (map:query user-map :name "Unknown"))
(print "Email (missing): ")
(println (map:query user-map :email "no-email@example.com"))
(println "")

(println "Map:select: select subset of keys")
(define full-map {:name "Bob" :age 25 :city "NYC" :country "USA"})
(print "Selected: ")
(println (map:select full-map '(:name :age)))
(println "")

(println "Map:update: update value with function")
(define age-map {:age 30})
(print "Incremented age: ")
(println (map:update age-map :age (lambda (x) (+ x 1))))
(println "")

(println "Map:from-entries: build map from pairs")
(print "From entries: ")
(println (map:from-entries '((:a 1) (:b 2) (:c 3))))
(println "")

(println "Map:filter: filter by predicate")
(define numbers-map {:a 1 :b 2 :c 3 :d 4})
(print "Even values only: ")
(println (map:filter (lambda (entry) (even? (car (cdr entry)))) numbers-map))
(println "")

(println "Map:map-values: transform all values")
(print "Doubled values: ")
(println (map:map-values (lambda (v) (* v 2)) numbers-map))
(println "")

(println "=== Demo Complete ===")
