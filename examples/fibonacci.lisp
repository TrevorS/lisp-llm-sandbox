;; Fibonacci Sequence
;; Multiple implementations showing different approaches

;; =============================================================================
;; Classic Recursive Fibonacci
;; =============================================================================
;; Simple but exponential time complexity O(2^n)
(define (fib-recursive n)
  (if (< n 2)
      n
      (+ (fib-recursive (- n 1))
         (fib-recursive (- n 2)))))

;; =============================================================================
;; Tail-Recursive Fibonacci (with TCO)
;; =============================================================================
;; Uses accumulator pattern, linear time O(n)
(define (fib-iter n a b count)
  (if (= count 0)
      a
      (fib-iter n b (+ a b) (- count 1))))

(define (fib-tail n)
  (fib-iter n 0 1 n))

;; =============================================================================
;; Generate Fibonacci Sequence as List
;; =============================================================================
(define (fib-list n)
  (if (<= n 0)
      '()
      (if (= n 1)
          '(0)
          (if (= n 2)
              '(0 1)
              (let ((prev (fib-list (- n 1))))
                (append prev
                        (list (+ (last prev)
                                 (last (reverse (cdr (reverse prev))))))))))))

;; =============================================================================
;; Examples
;; =============================================================================

(println "Fibonacci - Recursive (slow for large n):")
(println (fib-recursive 10))  ;; 55

(println "Fibonacci - Tail Recursive (fast, uses TCO):")
(println (fib-tail 10))  ;; 55
(println (fib-tail 100)) ;; Large Fibonacci number

(println "First 15 Fibonacci numbers:")
(println (fib-list 15))

;; Calculate multiple Fibonacci numbers
(define fib-nums (map fib-tail (range 0 20)))
(println "Fibonacci 0-19:")
(println fib-nums)
