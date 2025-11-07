;; Quicksort Algorithm
;; Classic divide-and-conquer sorting algorithm

(define (quicksort lst)
  (if (empty? lst)
      '()
      (append
        ;; Sort smaller elements
        (quicksort (filter (lambda (x) (< x (car lst))) (cdr lst)))
        ;; Add pivot
        (append
          (list (car lst))
          ;; Sort larger elements
          (quicksort (filter (lambda (x) (>= x (car lst))) (cdr lst)))))))

;; Example usage
(define unsorted '(3 1 4 1 5 9 2 6 5 3 5))
(define sorted (quicksort unsorted))

(println "Original list:")
(println unsorted)
(println "Sorted list:")
(println sorted)
