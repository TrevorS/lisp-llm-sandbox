;;; Examples of Maps and JSON module usage
;;;
;;; This demonstrates the new map data type and JSON encoding/decoding functionality

(println "=== Map Literals and Keywords ===")

;; Keywords are self-evaluating symbols starting with :
(println :name)           ; => :name
(println :age)            ; => :age

;; Map literals use {:key value} syntax
(define person {:name "Alice" :age 30 :city "NYC"})
(println person)          ; => {:age 30 :city "NYC" :name "Alice"}

(println "\n=== Map Operations ===")

;; Get value by key
(println (map-get person :name))        ; => "Alice"
(println (map-get person :missing nil)) ; => nil (with default)

;; Set creates a new map (immutable)
(define updated (map-set person :age 31))
(println updated)                        ; => {:age 31 :city "NYC" :name "Alice"}
(println person)                         ; Original unchanged

;; Check if key exists
(println (map-has? person :name))        ; => #t
(println (map-has? person :missing))     ; => #f

;; Get keys, values, and entries
(println (map-keys person))              ; => (:age :city :name)
(println (map-values person))            ; => (30 "NYC" "Alice")
(println (map-entries person))           ; => ((:age 30) (:city "NYC") (:name "Alice"))

;; Merge maps
(define extra {:country "USA" :active #t})
(define merged (map-merge person extra))
(println merged)                         ; => {:active #t :age 30 :city "NYC" :country "USA" :name "Alice"}

;; Remove key
(define without-age (map-remove person :age))
(println without-age)                    ; => {:city "NYC" :name "Alice"}

;; Check size and emptiness
(println (map-size person))              ; => 3
(println (map-empty? person))            ; => #f
(println (map-empty? {}))                ; => #t

(println "\n=== JSON Encoding ===")

;; Encode map to JSON
(define json-str (json:encode person))
(println json-str)                       ; => {"age":30,"city":"NYC","name":"Alice"}

;; Encode lists
(println (json:encode '(1 2 3)))        ; => [1,2,3]

;; Encode complex structures
(define complex-data {
  :user {:name "Bob" :email "bob@example.com"}
  :tags '("lisp" "rust" "json")
  :active #t
  :count 42
})
(println (json:encode complex-data))

;; Pretty print JSON
(println (json:pretty complex-data))

(println "\n=== JSON Decoding ===")

;; Decode JSON to maps and lists
(define decoded (json:decode "{\"x\":1,\"y\":2}"))
(println decoded)                        ; => {:x 1 :y 2}

(define arr (json:decode "[1,2,3,4,5]"))
(println arr)                            ; => (1 2 3 4 5)

(define null-val (json:decode "null"))
(println null-val)                       ; => nil

(println "\n=== Practical Example: API Response ===")

;; Simulate an API response
(define api-response (json:decode "{
  \"status\": \"success\",
  \"data\": {
    \"users\": [
      {\"id\": 1, \"name\": \"Alice\", \"role\": \"admin\"},
      {\"id\": 2, \"name\": \"Bob\", \"role\": \"user\"}
    ],
    \"total\": 2
  }
}"))

(println "API Status:" (map-get api-response :status))
(define data (map-get api-response :data))
(define users (map-get data :users))
(println "Total users:" (map-get data :total))
(println "First user:" (nth 0 users))
(println "First user name:" (map-get (nth 0 users) :name))

(println "\n=== Combining with Higher-Order Functions ===")

;; Map over list of maps
(define people '(
  {:name "Alice" :age 30}
  {:name "Bob" :age 25}
  {:name "Charlie" :age 35}
))

;; Extract names
(define names (map (lambda (p) (map-get p :name)) people))
(println "Names:" names)                 ; => ("Alice" "Bob" "Charlie")

;; Filter by age
(define over-30 (filter (lambda (p) (> (map-get p :age) 30)) people))
(println "Over 30:" over-30)            ; => ({:age 35 :name "Charlie"})

;; Build a map from list
(define coords '((:x 10) (:y 20) (:z 30)))
(define point (reduce
  (lambda (m pair) (map-set m (nth 0 pair) (nth 1 pair)))
  {}
  coords))
(println "Point:" point)                 ; => {:x 10 :y 20 :z 30}

(println "\nMaps and JSON: Complete!")
