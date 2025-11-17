;; ABOUTME: Core standard library - Higher-order functions, list utilities, and map helpers
;; This library provides foundational functional programming tools

;; ============================================================================
;; Higher-Order Functions
;; ============================================================================

;;; Apply function to each element, returning new list.
;;;
;;; **Parameters:**
;;; - f: Function to apply to each element
;;; - lst: Input list
;;;
;;; **Returns:** New list with f applied to each element
;;;
;;; **Time Complexity:** O(n) where n is list length
;;;
;;; **Examples:**
;;; - (map (lambda (x) (* x 2)) '(1 2 3)) => (2 4 6)
;;; - (map (lambda (x) (+ x 1)) '(0 1 2)) => (1 2 3)
;;;
;;; **Notes:** Uses tail call optimization for efficiency. Preserves list structure.
(define (map f lst)
  (if (empty? lst)
      '()
      (cons (f (car lst))
            (map f (cdr lst)))))

;;; Keep only elements satisfying predicate.
;;;
;;; **Parameters:**
;;; - pred: Predicate function returning boolean
;;; - lst: Input list
;;;
;;; **Returns:** New list containing only elements where pred returns true
;;;
;;; **Time Complexity:** O(n) where n is list length
;;;
;;; **Examples:**
;;; - (filter (lambda (x) (> x 2)) '(1 2 3 4 5)) => (3 4 5)
;;; - (filter even? '(1 2 3 4 5)) => (2 4)
;;;
;;; **Notes:** Preserves element order. Short-circuits on first predicate evaluation.
(define (filter pred lst)
  (if (empty? lst)
      '()
      (if (pred (car lst))
          (cons (car lst) (filter pred (cdr lst)))
          (filter pred (cdr lst)))))

;;; Fold list using function, accumulating from init value (left-fold).
;;;
;;; **Parameters:**
;;; - f: Binary function (accumulator, element) -> new-accumulator
;;; - init: Initial accumulator value
;;; - lst: Input list
;;;
;;; **Returns:** Final accumulated value
;;;
;;; **Time Complexity:** O(n) where n is list length
;;;
;;; **Examples:**
;;; - (reduce + 0 '(1 2 3 4)) => 10
;;; - (reduce * 1 '(1 2 3 4)) => 24
;;; - (reduce (lambda (acc x) (cons x acc)) '() '(1 2 3)) => (3 2 1)
;;;
;;; **Notes:** Left-associative fold. Processes list from head to tail.
(define (reduce f init lst)
  (if (empty? lst)
      init
      (reduce f (f init (car lst)) (cdr lst))))

;;; Compose two functions: returns function that applies g then f.
;;;
;;; **Parameters:**
;;; - f: Outer function
;;; - g: Inner function
;;;
;;; **Returns:** New function that computes f(g(x))
;;;
;;; **Mathematical Notation:** (compose f g)(x) = f(g(x))
;;;
;;; **Examples:**
;;; - (define double (lambda (x) (* x 2)))
;;; - (define inc (lambda (x) (+ x 1)))
;;; - ((compose double inc) 5) => 12 [= double(inc(5)) = double(6)]
;;;
;;; **Notes:** Useful for building function pipelines. Can be chained: (compose f (compose g h)).
(define (compose f g)
  (lambda (x) (f (g x))))

;;; Partially apply function with first argument fixed.
;;;
;;; **Parameters:**
;;; - f: Binary function to partially apply
;;; - arg: First argument to bind
;;;
;;; **Returns:** New function that accepts second argument
;;;
;;; **Examples:**
;;; - (define add5 (partial + 5))
;;; - (add5 10) => 15
;;; - (define mult-by-2 (partial * 2))
;;; - (mult-by-2 3) => 6
;;;
;;; **Common Use Cases:** Creating specialized functions from generic ones, currying, building function factories.
(define (partial f arg)
  (lambda (x) (f arg x)))

;; ============================================================================
;; List Utilities
;; ============================================================================

;;; Internal helper for list reversal using accumulator.
;;;
;;; **Parameters:**
;;; - lst: Remaining list to reverse
;;; - acc: Accumulator list (reversed so far)
;;;
;;; **Returns:** Complete reversed list
;;;
;;; **Time Complexity:** O(n) where n is list length
;;;
;;; **Notes:** Uses tail recursion for efficiency. Accessed via reverse/1.
(define (reverse-helper lst acc)
  (if (empty? lst)
      acc
      (reverse-helper (cdr lst) (cons (car lst) acc))))

;;; Reverse a list.
;;;
;;; **Parameters:**
;;; - lst: Input list
;;;
;;; **Returns:** New list with elements in reverse order
;;;
;;; **Time Complexity:** O(n) where n is list length
;;;
;;; **Examples:**
;;; - (reverse '(1 2 3)) => (3 2 1)
;;; - (reverse '(a b c)) => (c b a)
;;; - (reverse '()) => ()
;;;
;;; **Notes:** Creates new list, doesn't modify original. Tail-recursive via helper function.
(define (reverse lst)
  (reverse-helper lst '()))

;;; Concatenate two lists.
;;;
;;; **Parameters:**
;;; - lst1: First list
;;; - lst2: Second list
;;;
;;; **Returns:** New list with all elements from lst1 followed by lst2
;;;
;;; **Time Complexity:** O(m) where m is length of lst1
;;;
;;; **Examples:**
;;; - (append '(1 2) '(3 4)) => (1 2 3 4)
;;; - (append '(a) '(b c)) => (a b c)
;;; - (append '() '(1 2)) => (1 2)
;;;
;;; **Notes:** Second list is returned as-is; first list is copied. Linear in length of first argument.
(define (append lst1 lst2)
  (if (empty? lst1)
      lst2
      (cons (car lst1) (append (cdr lst1) lst2))))

;;; Check if element exists in list.
;;;
;;; **Parameters:**
;;; - x: Element to search for (uses = for comparison)
;;; - lst: List to search
;;;
;;; **Returns:** #t if element found, #f otherwise
;;;
;;; **Time Complexity:** O(n) worst case, O(1) best case
;;;
;;; **Examples:**
;;; - (member 2 '(1 2 3)) => #t
;;; - (member 5 '(1 2 3)) => #f
;;; - (member 'b '(a b c)) => #t
;;;
;;; **Notes:** Uses equality (=) for comparison. Short-circuits on first match.
(define (member x lst)
  (if (empty? lst)
      #f
      (if (= x (car lst))
          #t
          (member x (cdr lst)))))

;;; Get element at zero-based index.
;;;
;;; **Parameters:**
;;; - n: Zero-based index
;;; - lst: List to access
;;;
;;; **Returns:** Element at index n
;;;
;;; **Time Complexity:** O(n) where n is the index
;;;
;;; **Error Conditions:**
;;; - Negative index: undefined behavior
;;; - Index >= list length: error (attempts to car on empty list)
;;;
;;; **Examples:**
;;; - (nth 0 '(a b c)) => a
;;; - (nth 1 '(1 2 3)) => 2
;;; - (nth 2 '(x y z)) => z
;;;
;;; **Notes:** Zero-based indexing. Suitable for small indices; consider converting to vector for frequent access.
(define (nth n lst)
  (if (= n 0)
      (car lst)
      (nth (- n 1) (cdr lst))))

;;; Get the last element of a list.
;;;
;;; **Parameters:**
;;; - lst: Non-empty list
;;;
;;; **Returns:** Last element of the list
;;;
;;; **Time Complexity:** O(n) where n is list length
;;;
;;; **Examples:**
;;; - (last '(1 2 3)) => 3
;;; - (last '(a b c d)) => d
;;; - (last '(x)) => x
;;;
;;; **Error Conditions:**
;;; - Empty list: error (cdr of empty list)
;;;
;;; **Notes:** Must traverse entire list. For repeated access, reverse and use car.
(define (last lst)
  (if (empty? (cdr lst))
      (car lst)
      (last (cdr lst))))

;;; Get first n elements of a list.
;;;
;;; **Parameters:**
;;; - n: Number of elements to take
;;; - lst: Input list
;;;
;;; **Returns:** New list with first n elements (or entire list if n > length)
;;;
;;; **Time Complexity:** O(n) where n is the number to take
;;;
;;; **Examples:**
;;; - (take 2 '(1 2 3 4)) => (1 2)
;;; - (take 3 '(a b c d e)) => (a b c)
;;; - (take 0 '(1 2 3)) => ()
;;;
;;; **Notes:** Returns shorter list if requested number exceeds list length.
(define (take n lst)
  (if (or (= n 0) (empty? lst))
      '()
      (cons (car lst) (take (- n 1) (cdr lst)))))

;;; Skip first n elements of a list.
;;;
;;; **Parameters:**
;;; - n: Number of elements to skip
;;; - lst: Input list
;;;
;;; **Returns:** New list with first n elements removed
;;;
;;; **Time Complexity:** O(n) where n is the number to drop
;;;
;;; **Examples:**
;;; - (drop 2 '(1 2 3 4)) => (3 4)
;;; - (drop 1 '(a b c)) => (b c)
;;; - (drop 3 '(1 2 3)) => ()
;;;
;;; **Notes:** Returns empty list if n >= list length.
(define (drop n lst)
  (if (or (= n 0) (empty? lst))
      lst
      (drop (- n 1) (cdr lst))))

;;; Combine two lists into pairs.
;;;
;;; **Parameters:**
;;; - lst1: First list
;;; - lst2: Second list
;;;
;;; **Returns:** List of pairs [element1, element2]
;;;
;;; **Time Complexity:** O(min(n, m)) where n, m are list lengths
;;;
;;; **Examples:**
;;; - (zip '(1 2 3) '(a b c)) => ((1 a) (2 b) (3 c))
;;; - (zip '(x y) '(10 20)) => ((x 10) (y 20))
;;; - (zip '() '(1 2)) => ()
;;;
;;; **Notes:** Stops when shorter list ends. Use take/drop to pad lists.
(define (zip lst1 lst2)
  (if (or (empty? lst1) (empty? lst2))
      '()
      (cons (list (car lst1) (car lst2))
            (zip (cdr lst1) (cdr lst2)))))

;; ============================================================================
;; Map Utilities
;; ============================================================================

;;; Query nested map using keyword path list.
;;;
;;; **Parameters:**
;;; - data: Map or nested structure
;;; - path: List of keywords representing path
;;;
;;; **Returns:** Value at path, or nil if not found
;;;
;;; **Time Complexity:** O(n) where n is path depth
;;;
;;; **Examples:**
;;; - (map:query {:user {:name "Alice"}} '(:user :name)) => "Alice"
;;; - (map:query {:x {:y {:z 42}}} '(:x :y :z)) => 42
;;; - (map:query {:a 1} '(:b :c)) => nil
;;;
;;; **Notes:** Returns nil for missing keys or non-map values in path.
(define (map:query data path)
  (if (empty? path)
      data
      (if (map? data)
          (map:query (map-get data (car path) nil) (cdr path))
          nil)))

;;; Select subset of map by list of keys.
;;;
;;; **Parameters:**
;;; - m: Source map
;;; - keys: List of keywords to include
;;;
;;; **Returns:** New map with only specified keys
;;;
;;; **Time Complexity:** O(k) where k is number of keys
;;;
;;; **Examples:**
;;; - (map:select {:x 1 :y 2 :z 3} '(:x :z)) => {:x 1 :z 3}
;;; - (map:select {:a 1 :b 2} '(:a :c)) => {:a 1}
;;;
;;; **Notes:** Ignores keys not present in source map.
(define (map:select m keys)
  (reduce (lambda (acc key)
            (if (map-has? m key)
                (map-set acc key (map-get m key))
                acc))
          {}
          keys))

;;; Update map value by applying function.
;;;
;;; **Parameters:**
;;; - m: Map to update
;;; - key: Keyword key to update
;;; - f: Function to apply to current value
;;;
;;; **Returns:** New map with updated value
;;;
;;; **Time Complexity:** O(1)
;;;
;;; **Examples:**
;;; - (map:update {:count 5} :count (lambda (x) (+ x 1))) => {:count 6}
;;; - (map:update {:n 10} :n (lambda (x) (* x 2))) => {:n 20}
;;;
;;; **Notes:** Returns original map if key doesn't exist.
(define (map:update m key f)
  (if (map-has? m key)
      (map-set m key (f (map-get m key)))
      m))

;;; Build map from list of (key value) pairs.
;;;
;;; **Parameters:**
;;; - entries: List of (:key value) pairs
;;;
;;; **Returns:** New map
;;;
;;; **Time Complexity:** O(n) where n is number of entries
;;;
;;; **Examples:**
;;; - (map:from-entries '((:x 1) (:y 2))) => {:x 1 :y 2}
;;; - (map:from-entries '()) => {}
;;;
;;; **Notes:** Inverse of map-entries. Later entries override earlier ones.
(define (map:from-entries entries)
  (reduce (lambda (acc entry)
            (map-set acc (car entry) (car (cdr entry))))
          {}
          entries))

;;; Filter map by predicate function.
;;;
;;; **Parameters:**
;;; - pred: Function (key value) -> boolean
;;; - m: Map to filter
;;;
;;; **Returns:** New map with entries where predicate is true
;;;
;;; **Time Complexity:** O(n) where n is number of entries
;;;
;;; **Examples:**
;;; - (map:filter (lambda (k v) (> v 10)) {:x 5 :y 15 :z 20}) => {:y 15 :z 20}
;;; - (map:filter (lambda (k v) (= k :name)) {:name "Alice" :age 30}) => {:name "Alice"}
;;;
;;; **Notes:** Predicate receives both key and value.
(define (map:filter pred m)
  (map:from-entries
    (filter (lambda (entry)
              (pred (car entry) (car (cdr entry))))
            (map-entries m))))

;;; Transform all values in map using function.
;;;
;;; **Parameters:**
;;; - f: Function to apply to each value
;;; - m: Map to transform
;;;
;;; **Returns:** New map with transformed values
;;;
;;; **Time Complexity:** O(n) where n is number of entries
;;;
;;; **Examples:**
;;; - (map:map-values (lambda (x) (* x 2)) {:x 1 :y 2}) => {:x 2 :y 4}
;;; - (map:map-values string-upper {:a "hello" :b "world"}) => {:a "HELLO" :b "WORLD"}
;;;
;;; **Notes:** Keys remain unchanged, only values are transformed.
(define (map:map-values f m)
  (map:from-entries
    (map (lambda (entry)
           (list (car entry) (f (car (cdr entry)))))
         (map-entries m))))

;; Core library loaded
