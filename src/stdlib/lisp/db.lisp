;;; Database Module - Simplified implementation
;;;
;;; High-level database abstractions built on Rust primitives.
;;; Core functions: sqlite:spec, db:connect, db:disconnect, db:execute, db:select

;; ============================================================================
;; Backend Constructors
;; ============================================================================

;;; Create a SQLite connection specification
(define (sqlite:spec path)
  {:backend "sqlite" :path path})

;; ============================================================================
;; Connection Management
;; ============================================================================

;;; Open a database connection
(define (db:connect spec)
  (db:open spec))

;;; Close a database connection
(define (db:disconnect conn)
  (db:close conn))

;; ============================================================================
;; Query Execution Wrappers
;; ============================================================================

;;; Execute SQL statement (INSERT, UPDATE, DELETE, CREATE)
(define (db:execute conn sql params)
  (db:exec conn sql params))

;;; Execute SELECT query
(define (db:select conn sql params)
  (db:query conn sql params))

;; ============================================================================
;; Helper Functions
;; ============================================================================

;;; Convert keyword to column name
(define (key->column key)
  (keyword->string key))

;;; Get first row from results
(define (db:first results)
  (if (empty? results)
      nil
      (car results)))

;;; Extract column values from all rows
(define (db:pluck results column)
  (map (lambda (row) (map-get row column)) results))

;; ============================================================================
;; Query Builders
;; ============================================================================

;;; Insert a row into a table
(define (db:insert conn table data)
  "Insert a row into a table.

**Parameters:**
- conn: Database connection map
- table: Table name (String)
- data: Map of column names to values (e.g., {:id 1 :name \"Alice\"})

**Returns:** Number of rows inserted (typically 1)

**Time Complexity:** O(n) where n is the number of columns

**Examples:**
- (db:insert conn \"users\" {:id 1 :name \"Alice\" :age 30})
- (db:insert conn \"products\" {:name \"Widget\" :price 9.99})

**Notes:** Uses parameterized queries to prevent SQL injection."
  (begin
    (define keys (map-keys data))
    (define col-names (map keyword->string keys))
    (define values (map (lambda (k) (map-get data k)) keys))
    (define placeholders (map (lambda (_) "?") keys))
    (define columns-str (string-join col-names ", "))
    (define placeholders-str (string-join placeholders ", "))
    (define sql (string-append "INSERT INTO " table " (" columns-str ") VALUES (" placeholders-str ")"))
    (db:exec conn sql values)))

;;; Update rows in a table
(define (db:update conn table data where-map)
  "Update rows in a table.

**Parameters:**
- conn: Database connection map
- table: Table name (String)
- data: Map of column names to new values
- where-map: Map of column names to filter values

**Returns:** Number of rows updated

**Time Complexity:** O(n + m) where n is columns to update, m is WHERE conditions

**Examples:**
- (db:update conn \"users\" {:age 31} {:id 1})
- (db:update conn \"products\" {:price 12.99} {:name \"Widget\"})

**Notes:** All WHERE conditions are AND-ed together. Empty where-map is rejected to prevent updating all rows."
  (begin
    ;; Safety check: prevent accidental update of all rows
    (if (= (map-size where-map) 0)
        (error "db:update: Empty where-map would update ALL rows. Use db:exec for UPDATE without WHERE if intentional.")
        (begin
          (define set-keys (map-keys data))
          (define set-names (map keyword->string set-keys))
          (define set-values (map (lambda (k) (map-get data k)) set-keys))
          (define set-clauses (map (lambda (name) (string-append name " = ?")) set-names))
          (define set-str (string-join set-clauses ", "))

          (define where-keys (map-keys where-map))
          (define where-names (map keyword->string where-keys))
          (define where-values (map (lambda (k) (map-get where-map k)) where-keys))
          (define where-clauses (map (lambda (name) (string-append name " = ?")) where-names))
          (define where-str (string-join where-clauses " AND "))

          (define all-values (append set-values where-values))
          (define sql (string-append "UPDATE " table " SET " set-str " WHERE " where-str))
          (db:exec conn sql all-values)))))

;;; Delete rows from a table
(define (db:delete conn table where-map)
  "Delete rows from a table.

**Parameters:**
- conn: Database connection map
- table: Table name (String)
- where-map: Map of column names to filter values

**Returns:** Number of rows deleted

**Time Complexity:** O(n) where n is the number of WHERE conditions

**Examples:**
- (db:delete conn \"users\" {:id 1})
- (db:delete conn \"products\" {:price 0})

**Notes:** All WHERE conditions are AND-ed together. Empty where-map is rejected to prevent deleting all rows."
  (begin
    ;; Safety check: prevent accidental deletion of all rows
    (if (= (map-size where-map) 0)
        (error "db:delete: Empty where-map would delete ALL rows. Use db:exec for DELETE without WHERE if intentional.")
        (begin
          (define keys (map-keys where-map))
          (define col-names (map keyword->string keys))
          (define values (map (lambda (k) (map-get where-map k)) keys))
          (define clauses (map (lambda (name) (string-append name " = ?")) col-names))
          (define where-str (string-join clauses " AND "))
          (define sql (string-append "DELETE FROM " table " WHERE " where-str))
          (db:exec conn sql values)))))

;;; Find rows in a table
(define (db:find conn table columns where-map)
  "Find rows in a table.

**Parameters:**
- conn: Database connection map
- table: Table name (String)
- columns: List of column names to select, or \"*\" for all
- where-map: Map of column names to filter values (empty map returns all rows)

**Returns:** List of row maps matching the WHERE conditions

**Time Complexity:** O(n + m) where n is columns, m is WHERE conditions

**Examples:**
- (db:find conn \"users\" '(\"id\" \"name\") {:age 30})
- (db:find conn \"products\" \"*\" {:price 9.99})
- (db:find conn \"users\" \"*\" {})  ; Get all users

**Notes:** All WHERE conditions are AND-ed together."
  (begin
    (define cols-str (if (string? columns)
                         columns
                         (string-join columns ", ")))

    (define keys (map-keys where-map))
    (define values (map (lambda (k) (map-get where-map k)) keys))

    (define sql (if (empty? keys)
                    (string-append "SELECT " cols-str " FROM " table)
                    (begin
                      (define col-names (map keyword->string keys))
                      (define clauses (map (lambda (name) (string-append name " = ?")) col-names))
                      (define where-str (string-join clauses " AND "))
                      (string-append "SELECT " cols-str " FROM " table " WHERE " where-str))))

    (db:query conn sql values)))

;;; Count rows in a table
(define (db:count conn table where-map)
  "Count rows in a table.

**Parameters:**
- conn: Database connection map
- table: Table name (String)
- where-map: Map of column names to filter values (empty map counts all rows)

**Returns:** Number of rows matching the WHERE conditions

**Time Complexity:** O(n) where n is the number of WHERE conditions

**Examples:**
- (db:count conn \"users\" {:age 30})
- (db:count conn \"products\" {})  ; Count all products

**Notes:** All WHERE conditions are AND-ed together."
  (begin
    (define keys (map-keys where-map))
    (define values (map (lambda (k) (map-get where-map k)) keys))

    (define sql (if (empty? keys)
                    (string-append "SELECT COUNT(*) as count FROM " table)
                    (begin
                      (define col-names (map keyword->string keys))
                      (define clauses (map (lambda (name) (string-append name " = ?")) col-names))
                      (define where-str (string-join clauses " AND "))
                      (string-append "SELECT COUNT(*) as count FROM " table " WHERE " where-str))))

    (define result (db:query conn sql values))
    (define first-row (db:first result))
    (if (nil? first-row)
        0
        (map-get first-row :count))))

;;; Transaction management

(define (db:begin conn)
  "Begin a database transaction.

**Parameters:**
- conn: Database connection map

**Returns:** Number of rows affected (typically 0)

**Time Complexity:** O(1)

**Examples:**
- (db:begin conn)

**Notes:** Must be followed by db:commit or db:rollback. Transactions provide atomicity -
either all operations succeed or all are rolled back."
  (db:exec conn "BEGIN TRANSACTION" '()))

(define (db:commit conn)
  "Commit the current transaction.

**Parameters:**
- conn: Database connection map

**Returns:** Number of rows affected (typically 0)

**Time Complexity:** O(1)

**Examples:**
- (db:begin conn)
- (db:insert conn \"users\" {:id 1 :name \"Alice\"})
- (db:commit conn)

**Notes:** Finalizes all changes made since db:begin. If commit fails, changes are rolled back."
  (db:exec conn "COMMIT" '()))

(define (db:rollback conn)
  "Roll back the current transaction.

**Parameters:**
- conn: Database connection map

**Returns:** Number of rows affected (typically 0)

**Time Complexity:** O(1)

**Examples:**
- (db:begin conn)
- (db:insert conn \"users\" {:id 1 :name \"Alice\"})
- (db:rollback conn)  ; Alice was not inserted

**Notes:** Discards all changes made since db:begin. Use when an error occurs or you want to abort."
  (db:exec conn "ROLLBACK" '()))

;;; Convenience macros

(defmacro with-db (binding . body)
  "Execute body with automatic connection cleanup.

**Parameters:**
- binding: (conn-var spec) where conn-var is the variable name and spec is the connection spec
- body: Expressions to execute with the connection

**Returns:** Result of the last expression in body

**Examples:**
- (with-db (conn (sqlite:spec \"users.db\"))
    (db:query conn \"SELECT * FROM users\" '())
    (db:insert conn \"users\" {:id 1 :name \"Alice\"}))
  ; Connection automatically closed after body

**Notes:** Ensures db:close is called even if body completes normally.
Recommended pattern for all database operations to prevent resource leaks."
  `(begin
     (define ,(car binding) (db:connect ,(car (cdr binding))))
     (define __db_result (begin ,@body))
     (db:close ,(car binding))
     __db_result))

(defmacro with-transaction (conn . body)
  "Execute body within a transaction with automatic commit/rollback.

**Parameters:**
- conn: Database connection variable
- body: Expressions to execute within the transaction

**Returns:** Result of the last expression in body

**Examples:**
- (with-db (conn (sqlite:spec \"users.db\"))
    (with-transaction conn
      (db:insert conn \"users\" {:id 1 :name \"Alice\"})
      (db:insert conn \"orders\" {:user_id 1 :item \"Widget\"})))
  ; Both inserts committed atomically, or both rolled back on error

**Notes:** Automatically commits if body succeeds, rolls back if body produces an error.
Can be nested with with-db for complete resource safety."
  `(begin
     (db:begin ,conn)
     (define __tx_result (begin ,@body))
     (if (error? __tx_result)
         (begin
           (db:rollback ,conn)
           __tx_result)
         (begin
           (db:commit ,conn)
           __tx_result))))
