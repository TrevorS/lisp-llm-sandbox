;; Quicksort Algorithm
;; Classic divide-and-conquer sorting algorithm

(define (quicksort lst)
  (if (empty? lst)
      '()
      (let ((pivot (car lst))
            (rest (cdr lst)))
        (let ((smaller (quicksort (filter (lambda (x) (< x pivot)) rest)))
              (larger (quicksort (filter (lambda (x) (>= x pivot)) rest))))
          (append smaller (cons pivot larger))))))

;; Example usage
(define unsorted '(3 1 4 1 5 9 2 6 5 3 5))
(define sorted (quicksort unsorted))

(println "Original list:")
(println unsorted)
(println "Sorted list:")
(println sorted)
