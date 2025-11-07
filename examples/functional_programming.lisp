;; Functional Programming Patterns
;; Demonstrating map, filter, reduce, and higher-order functions

;; =============================================================================
;; Map, Filter, Reduce
;; =============================================================================

(define numbers '(1 2 3 4 5 6 7 8 9 10))

;; Map: Transform each element
(define doubles (map (lambda (x) (* x 2)) numbers))
(println "Doubled:")
(println doubles)

;; Filter: Keep only elements matching predicate
(define evens (filter (lambda (x) (even? x)) numbers))
(println "Even numbers:")
(println evens)

;; Reduce: Accumulate values
(define sum-all (reduce + 0 numbers))
(println "Sum of all numbers:")
(println sum-all)

;; =============================================================================
;; Function Composition
;; =============================================================================

;; Define some simple functions
(define (square x) (* x x))
(define (inc x) (+ x 1))
(define (double x) (* x 2))

;; Compose them
(define inc-then-square (compose square inc))
(define double-then-inc (compose inc double))

(println "Compose examples:")
(println (inc-then-square 5))    ;; (5 + 1)^2 = 36
(println (double-then-inc 5))    ;; (5 * 2) + 1 = 11

;; =============================================================================
;; Closures and Partial Application
;; =============================================================================

;; Function that returns a function (closure)
(define (make-multiplier n)
  (lambda (x) (* n x)))

(define times-10 (make-multiplier 10))
(define times-100 (make-multiplier 100))

(println "Closure examples:")
(println (times-10 5))   ;; 50
(println (times-100 5))  ;; 500

;; Partial application
(define add-5 (partial + 5))
(println "Partial application:")
(println (add-5 10))  ;; 15

;; =============================================================================
;; Chaining Operations
;; =============================================================================

;; Get sum of squares of even numbers
(define result
  (reduce +
          0
          (map square
               (filter even? numbers))))

(println "Sum of squares of even numbers:")
(println result)  ;; 2^2 + 4^2 + 6^2 + 8^2 + 10^2 = 220

;; =============================================================================
;; Predicates
;; =============================================================================

(define (positive? x) (> x 0))
(define (negative? x) (< x 0))

(define mixed '(-5 -3 0 2 4))

(println "All positive?")
(println (all positive? numbers))  ;; #t

(println "Any negative in mixed?")
(println (any negative? mixed))  ;; #t

(println "Count positives in mixed:")
(println (count positive? mixed))  ;; 2
