duckdb --version
sqlite3 --version

if [ -f duckdb_test.db ]; then
    rm duckdb_test.db
fi

if [ -f sqlite_test.db ]; then
    rm sqlite_test.db
fi

# Create a DuckDB database and populate it with data from a Parquet file
duckdb duckdb_test.db <<EOF
CREATE TABLE test_table AS SELECT * FROM read_parquet('test_data.parquet');
-- DuckDB does not benefit from composite indexes, so we create them separately
-- https://github.com/duckdb/duckdb/issues/17306
CREATE INDEX idx_test_table_field1 ON test_table(Field1);
CREATE INDEX idx_test_table_field2 ON test_table(Field2);
EOF

# Create a SQLite database and populate it with data from a Parquet file
duckdb duckdb_test.db <<EOF
INSTALL sqlite;
LOAD sqlite;
ATTACH 'sqlite_test.db' AS sqlite_db (TYPE sqlite);
CREATE TABLE sqlite_db.test_table AS
SELECT * FROM read_parquet('test_data.parquet');

CREATE TABLE sqlite_db.test_table2 AS
SELECT * FROM read_parquet('test_data.parquet');
EOF

# Create indexes using sqlite3 directly
sqlite3 sqlite_test.db <<EOF
-- Create separate indexes for test_table
CREATE INDEX idx_test_table_field1 ON test_table(Field1);
CREATE INDEX idx_test_table_field2 ON test_table(Field2);

-- Create composite index for test_table2
CREATE INDEX idx_test_table2_composite ON test_table2(Field1, Field2);
EOF

echo "Database setup complete with indexes created"

# Print schema and record counts
echo "=== DUCKDB DATABASE SCHEMA, INDEXES, AND RECORD COUNTS ==="
duckdb duckdb_test.db <<EOF
DESCRIBE test_table;
SELECT * FROM duckdb_indexes WHERE table_name = 'test_table';
SELECT 'test_table record count:', COUNT(*) FROM test_table;
EOF

echo ""

echo "=== SQLITE DATABASE SCHEMA AND RECORD COUNTS ==="
sqlite3 sqlite_test.db <<EOF
.schema test_table
.schema test_table2
SELECT 'test_table record count:', COUNT(*) FROM test_table;
SELECT 'test_table2 record count:', COUNT(*) FROM test_table2;
EOF
