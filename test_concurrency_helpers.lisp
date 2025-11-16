;; Test script for concurrency helper functions

(println "Testing concurrency helpers...")
(println "")

;; Test 1: parallel-map with simple computation
(println "1. Testing parallel-map with simple computation:")
(define result1 (parallel-map (lambda (x) (* x 2)) '(1 2 3 4 5)))
(print "   Result: ")
(println result1)
(println "   Expected: (2 4 6 8 10)")
(println "")

;; Test 2: pmap alias
(println "2. Testing pmap (alias for parallel-map):")
(define result2 (pmap square '(1 2 3 4)))
(print "   Result: ")
(println result2)
(println "   Expected: (1 4 9 16)")
(println "")

;; Test 3: parallel-map-link with success cases
(println "3. Testing parallel-map-link with all successes:")
(define result3 (parallel-map-link (lambda (x) (+ x 10)) '(1 2 3)))
(print "   Result: ")
(println result3)
(println "   Expected: List of maps with 'ok' keys")
(println "")

;; Test 4: parallel-map-link with error case
(println "4. Testing parallel-map-link with division by zero:")
(define result4 (parallel-map-link (lambda (x) (/ 10 x)) '(2 1 0)))
(print "   Result: ")
(println result4)
(println "   Expected: Two successes, one error for division by zero")
(println "")

;; Test 5: fan-out - apply multiple functions to one value
(println "5. Testing fan-out:")
(define result5 (fan-out 5 (list square cube (lambda (x) (* x 10)))))
(print "   Result: ")
(println result5)
(println "   Expected: (25 125 50)")
(println "")

;; Test 6: parallel-pipeline - independent tasks
(println "6. Testing parallel-pipeline:")
(define result6 (parallel-pipeline
  (list
    (lambda () (+ 1 2 3))
    (lambda () (* 4 5))
    (lambda () (- 10 3)))))
(print "   Result: ")
(println result6)
(println "   Expected: List of maps with {:ok 6}, {:ok 20}, {:ok 7}")
(println "")

;; Test 7: parallel-for-each (side effects)
(println "7. Testing parallel-for-each (side effects):")
(parallel-for-each
  (lambda (x) (begin (print "   Processing: ") (println x)))
  '(1 2 3))
(println "   (Should see 3 processing messages above)")
(println "")

(println "All concurrency helper tests completed!")
