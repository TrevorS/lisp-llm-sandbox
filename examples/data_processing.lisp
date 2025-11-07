;; Data Processing Examples
;; Working with lists and transformations

;; =============================================================================
;; Sample Data
;; =============================================================================

(define students-scores '(85 92 78 95 88 76 91 89 94 82))
(define temperatures '(72 68 75 70 73 69 74 71 76 77))

;; =============================================================================
;; Statistical Functions
;; =============================================================================

(define (average lst)
  (/ (sum lst) (length lst)))

(define (min-of-list lst)
  (reduce min (car lst) (cdr lst)))

(define (max-of-list lst)
  (reduce max (car lst) (cdr lst)))

;; =============================================================================
;; Analysis
;; =============================================================================

(println "Student Scores Analysis:")
(println "========================")
(println "All scores:")
(println students-scores)
(println "Average score:")
(println (average students-scores))
(println "Highest score:")
(println (max-of-list students-scores))
(println "Lowest score:")
(println (min-of-list students-scores))

;; Students who passed (>= 80)
(define passing (filter (lambda (x) (>= x 80)) students-scores))
(println "Passing scores (>=80):")
(println passing)
(println "Pass rate:")
(println (/ (length passing) (length students-scores)))

;; =============================================================================
;; Temperature Analysis
;; =============================================================================

(println "")
(println "Temperature Analysis:")
(println "=====================")
(println "All temperatures:")
(println temperatures)
(println "Average temperature:")
(println (average temperatures))

;; Comfortable range (70-75)
(define comfortable
  (filter (lambda (t) (and (>= t 70) (<= t 75)))
          temperatures))
(println "Comfortable days (70-75):")
(println (length comfortable))

;; =============================================================================
;; Data Transformation
;; =============================================================================

;; Convert scores to letter grades
(define (to-letter-grade score)
  (if (>= score 90) "A"
    (if (>= score 80) "B"
      (if (>= score 70) "C"
        (if (>= score 60) "D"
          "F")))))

;; Note: This would create letter grades if we had strings working
;; (define grades (map to-letter-grade students-scores))

;; Normalize scores to 0-100 scale
(define (normalize score min max)
  (* 100 (/ (- score min) (- max min))))

(define min-score (min-of-list students-scores))
(define max-score (max-of-list students-scores))

(println "")
(println "Normalized Scores (0-100):")
(define normalized
  (map (lambda (s) (normalize s min-score max-score))
       students-scores))
(println normalized)

;; =============================================================================
;; List Manipulation
;; =============================================================================

(println "")
(println "List Manipulation:")
(println "==================")

;; Reverse a list
(define reversed-scores (reverse students-scores))
(println "Reversed scores:")
(println reversed-scores)

;; Get top 3 scores (would need sorting first in real scenario)
(println "First 3 scores:")
(println (take 3 students-scores))

;; Get last 3 scores
(println "Last 3 scores:")
(println (take 3 (reverse students-scores)))

;; Zip scores with positions
(define positions (range 1 11))
(define score-pairs (zip positions students-scores))
(println "Position-Score pairs:")
(println score-pairs)
