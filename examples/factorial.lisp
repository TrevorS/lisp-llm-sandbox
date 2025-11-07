;; Factorial Implementations
;; Demonstrating both recursive and tail-recursive approaches

;; Classic recursive factorial
;; factorial(n) = n * factorial(n-1), base case: factorial(0) = 1
(define (factorial n)
  (if (<= n 1)
      1
      (* n (factorial (- n 1)))))

;; Tail-recursive factorial with accumulator
;; Takes advantage of tail call optimization (TCO)
(define (factorial-tr n acc)
  (if (<= n 1)
      acc
      (factorial-tr (- n 1) (* acc n))))

(define (factorial-tail n)
  (factorial-tr n 1))

;; Example usage
(println "Factorial of 5:")
(println (factorial 5))

(println "Factorial of 10:")
(println (factorial 10))

(println "Factorial of 20 (using tail recursion):")
(println (factorial-tail 20))

;; The tail-recursive version can handle much larger inputs
;; without stack overflow thanks to TCO
(println "Factorial of 100 (tail-recursive, no stack overflow):")
(println (factorial-tail 100))
