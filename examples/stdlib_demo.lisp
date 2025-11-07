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

(println "=== Demo Complete ===")
