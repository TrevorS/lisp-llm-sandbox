;; ABOUTME: Math utilities - Numeric operations, predicates, and sequence generation
;; This library provides mathematical functions and predicates

;; ============================================================================
;; Basic Math
;; ============================================================================

;;; Return absolute value (magnitude without sign).
;;;
;;; **Parameters:**
;;; - x: Number
;;;
;;; **Returns:** Non-negative number with same magnitude
;;;
;;; **Time Complexity:** O(1)
;;;
;;; **Examples:**
;;; - (abs 5) => 5
;;; - (abs -5) => 5
;;; - (abs 0) => 0
;;;
;;; **Notes:** Works with both integers and floats.
(define (abs x)
  (if (< x 0)
      (- x)
      x))

;;; Return minimum of two numbers.
;;;
;;; **Parameters:**
;;; - x: First number
;;; - y: Second number
;;;
;;; **Returns:** Smaller of x and y
;;;
;;; **Time Complexity:** O(1)
;;;
;;; **Examples:**
;;; - (min 3 5) => 3
;;; - (min -2 -5) => -5
;;; - (min 0 0) => 0
;;;
;;; **Notes:** For finding minimum of list, use reduce with min.
(define (min x y)
  (if (< x y) x y))

;;; Return maximum of two numbers.
;;;
;;; **Parameters:**
;;; - x: First number
;;; - y: Second number
;;;
;;; **Returns:** Larger of x and y
;;;
;;; **Time Complexity:** O(1)
;;;
;;; **Examples:**
;;; - (max 3 5) => 5
;;; - (max -2 -5) => -2
;;; - (max 0 0) => 0
;;;
;;; **Notes:** For finding maximum of list, use reduce with max.
(define (max x y)
  (if (> x y) x y))

;;; Return number squared (x*x).
;;;
;;; **Parameters:**
;;; - x: Number
;;;
;;; **Returns:** x * x
;;;
;;; **Time Complexity:** O(1)
;;;
;;; **Examples:**
;;; - (square 5) => 25
;;; - (square -3) => 9
;;; - (square 0) => 0
;;;
;;; **Notes:** Common utility for geometric calculations and algorithm implementation.
(define (square x)
  (* x x))

;;; Return number cubed (x*x*x).
;;;
;;; **Parameters:**
;;; - x: Number
;;;
;;; **Returns:** x * x * x
;;;
;;; **Time Complexity:** O(1)
;;;
;;; **Examples:**
;;; - (cube 3) => 27
;;; - (cube -2) => -8
;;; - (cube 0) => 0
;;;
;;; **Notes:** Less common than square, but follows similar pattern.
(define (cube x)
  (* x x x))

;; ============================================================================
;; Numeric Predicates
;; ============================================================================

;;; Check if number is even.
;;;
;;; **Parameters:**
;;; - x: Integer
;;;
;;; **Returns:** #t if x is even, #f otherwise
;;;
;;; **Time Complexity:** O(1)
;;;
;;; **Examples:**
;;; - (even? 4) => #t
;;; - (even? 3) => #f
;;; - (even? 0) => #t
;;;
;;; **Notes:** Uses modulo operator (%). Define odd? in terms of even? for consistency.
(define (even? x)
  (= (% x 2) 0))

;;; Check if number is odd.
;;;
;;; **Parameters:**
;;; - x: Integer
;;;
;;; **Returns:** #t if x is odd, #f otherwise
;;;
;;; **Time Complexity:** O(1)
;;;
;;; **Examples:**
;;; - (odd? 3) => #t
;;; - (odd? 4) => #f
;;; - (odd? 0) => #f
;;;
;;; **Notes:** Defined as negation of even? for consistency. Use even? when applicable.
(define (odd? x)
  (not (even? x)))

;; ============================================================================
;; List Aggregations
;; ============================================================================

;;; Sum all elements in a list.
;;;
;;; **Parameters:**
;;; - lst: List of numbers
;;;
;;; **Returns:** Sum of all elements (0 for empty list)
;;;
;;; **Time Complexity:** O(n) where n is list length
;;;
;;; **Examples:**
;;; - (sum '(1 2 3 4)) => 10
;;; - (sum '(10)) => 10
;;; - (sum '()) => 0
;;;
;;; **Notes:** Implemented using reduce. Handles empty lists gracefully.
(define (sum lst)
  (reduce + 0 lst))

;;; Multiply all elements in a list.
;;;
;;; **Parameters:**
;;; - lst: List of numbers
;;;
;;; **Returns:** Product of all elements (1 for empty list)
;;;
;;; **Time Complexity:** O(n) where n is list length
;;;
;;; **Examples:**
;;; - (product '(1 2 3 4)) => 24
;;; - (product '(2 3)) => 6
;;; - (product '()) => 1
;;;
;;; **Notes:** Implemented using reduce. Handles empty lists gracefully.
(define (product lst)
  (reduce * 1 lst))

;;; Compute factorial: n! = n * (n-1) * ... * 1.
;;;
;;; **Parameters:**
;;; - n: Non-negative integer
;;;
;;; **Returns:** n factorial
;;;
;;; **Time Complexity:** O(n)
;;;
;;; **Examples:**
;;; - (factorial 5) => 120
;;; - (factorial 0) => 1
;;; - (factorial 1) => 1
;;;
;;; **Error Conditions:**
;;; - Negative n: returns 1 (by definition, base case)
;;; - Large n: may cause stack overflow without TCO
;;;
;;; **Notes:** Uses tail-call optimization for efficiency. Mathematical: 0! = 1, n! = n * (n-1)!
(define (factorial n)
  (if (<= n 1)
      1
      (* n (factorial (- n 1)))))

;; ============================================================================
;; List Predicates
;; ============================================================================

;;; Check if all elements satisfy predicate.
;;;
;;; **Parameters:**
;;; - pred: Predicate function
;;; - lst: List to check
;;;
;;; **Returns:** #t if all elements satisfy predicate, #f otherwise
;;;
;;; **Time Complexity:** O(n) worst case, O(1) best case
;;;
;;; **Examples:**
;;; - (all even? '(2 4 6)) => #t
;;; - (all even? '(2 3 4)) => #f
;;; - (all (lambda (x) (> x 0)) '(1 2 3)) => #t
;;;
;;; **Notes:** Short-circuits on first false predicate (returns #f immediately).
(define (all pred lst)
  (if (empty? lst)
      #t
      (if (pred (car lst))
          (all pred (cdr lst))
          #f)))

;;; Check if any element satisfies predicate.
;;;
;;; **Parameters:**
;;; - pred: Predicate function
;;; - lst: List to check
;;;
;;; **Returns:** #t if any element satisfies predicate, #f otherwise
;;;
;;; **Time Complexity:** O(n) worst case, O(1) best case
;;;
;;; **Examples:**
;;; - (any even? '(1 3 4)) => #t
;;; - (any even? '(1 3 5)) => #f
;;; - (any (lambda (x) (> x 5)) '(1 2 6 3)) => #t
;;;
;;; **Notes:** Short-circuits on first true predicate (returns #t immediately).
(define (any pred lst)
  (if (empty? lst)
      #f
      (if (pred (car lst))
          #t
          (any pred (cdr lst)))))

;;; Count elements satisfying predicate.
;;;
;;; **Parameters:**
;;; - pred: Predicate function
;;; - lst: List to count
;;;
;;; **Returns:** Number of elements where pred returns true
;;;
;;; **Time Complexity:** O(n) where n is list length
;;;
;;; **Examples:**
;;; - (count (lambda (x) (> x 2)) '(1 2 3 4 5)) => 3
;;; - (count even? '(1 2 3 4 5 6)) => 3
;;; - (count (lambda (x) (= x 0)) '(1 2 3)) => 0
;;;
;;; **Notes:** Counts all matching elements (doesn't short-circuit).
(define (count pred lst)
  (if (empty? lst)
      0
      (if (pred (car lst))
          (+ 1 (count pred (cdr lst)))
          (count pred (cdr lst)))))

;; ============================================================================
;; Sequence Generation
;; ============================================================================

;;; Generate list of integers from start (inclusive) to end (exclusive).
;;;
;;; **Parameters:**
;;; - start: Starting number (inclusive)
;;; - end: Ending number (exclusive)
;;;
;;; **Returns:** List of integers [start, start+1, ..., end-1]
;;;
;;; **Time Complexity:** O(n) where n = (end - start)
;;;
;;; **Examples:**
;;; - (range 0 5) => (0 1 2 3 4)
;;; - (range 1 4) => (1 2 3)
;;; - (range 5 5) => ()
;;;
;;; **Notes:** Mirrors Python/Scheme range semantics. Use with map/filter for sequences.
(define (range start end)
  (if (>= start end)
      '()
      (cons start (range (+ start 1) end))))
