# SQLite vs DuckDB Performance Comparison

Setup scripts and a Rust-based HTTP server to compare query performance between SQLite (using Turso's Limbo) and DuckDB on a dataset with 7.5 million rows and indexes.

- **SQLite** (via [Turso's Limbo](https://github.com/tursodatabase/limbo)) - A modern, high-performance SQLite implementation
- **DuckDB** - DuckDB is a fast analytical database system optimized for OLAP workloads

The comparison focuses on indexed lookups using VARCHAR columns.

## Architecture

- **HTTP Server**: Built with Axum, exposing two REST endpoints for query execution
- **Database Engines**
    - **DuckDB** (`1.3.1`): `duckdb_test.db` with `test_table` and individual indexes on `Field1` and `Field2`
    - **SQLite** (limbo `0.0.20`):
        - `test_table` in `sqlite_test.db` with separate indexes on `Field1` (`idx_test_table_field1`) and `Field2` (`idx_test_table_field2`)
        - `test_table2` in `sqlite_test.db` with a composite index on `(Field1, Field2)` (`idx_test_table2_composite`)
- **Connection Pooling**:
    - Uses `r2d2` for DuckDB connections. Pool size: 10 connections.
    - Uses `bb8` for SQLite (Limbo) connections. Pool size: 10 connections.
- **Load Testing**: OHA (HTTP load testing tool) for performance benchmarking

## Prerequisites

1. **DuckDB CLI**: [Installation Guide](https://duckdb.org/docs/installation/)
2. **SQLite3 CLI**: Usually pre-installed on macOS/Linux (https://sqlite.org/cli.html)
3. **OHA**: HTTP load testing tool (`cargo install oha` or `brew install oha`): https://github.com/hatoo/oha
4. **Rust**
5. **Make**

## Setup

1. Download Test Data (1.2GB)

```bash
wget https://spiceai-public-datasets.s3.us-east-1.amazonaws.com/debugging/test_data.parquet
```

Output:

```bash
--2025-06-29 23:04:22--  https://spiceai-public-datasets.s3.us-east-1.amazonaws.com/debugging/test_data.parquet
Resolving spiceai-public-datasets.s3.us-east-1.amazonaws.com (spiceai-public-datasets.s3.us-east-1.amazonaws.com)... 3.5.0.70, 3.5.16.3, 16.182.71.18, ...
Connecting to spiceai-public-datasets.s3.us-east-1.amazonaws.com (spiceai-public-datasets.s3.us-east-1.amazonaws.com)|3.5.0.70|:443... connected.
HTTP request sent, awaiting response... 200 OK
Length: 1261964038 (1.2G) [binary/octet-stream]
Saving to: ‘test_data.parquet’

test_data.parquet                                                                           100%[==========================================================================================================================================================================================================================================>]   1.17G  10.2MB/s    in 1m 41s  

2025-06-29 23:06:04 (11.9 MB/s) - ‘test_data.parquet’ saved [1261964038/1261964038]
```

2. Setup Test Databases

Populate test DuckDB and SQLite databases with 7.5M records and create indexes.This script will:

- Create `duckdb_test.db` with a single table and separate indexes on `Field1` and `Field2`
- Create `sqlite_test.db` with two tables:
  - `test_table`: separate indexes on `Field1` and `Field2`
  - `test_table2`: composite index on `(Field1, Field2)`

```bash
./setup.sh
```

Output:

```bash
v1.3.1 (Ossivalis) 2063dda3e6
3.43.2 2023-10-10 13:08:14 1b37c146ee9ebb7acd0160c0ab1fd11017a419fa8a3187386ed8cb32b709aapl (64-bit)
100% ▕████████████████████████████████████████████████████████████▏ 
100% ▕████████████████████████████████████████████████████████████▏ 
100% ▕████████████████████████████████████████████████████████████▏ 
100% ▕████████████████████████████████████████████████████████████▏ 
100% ▕████████████████████████████████████████████████████████████▏ 
Database setup complete with indexes created
=== DUCKDB DATABASE SCHEMA, INDEXES, AND RECORD COUNTS ===
┌─────────────┬──────────────┬─────────┬─────────┬─────────┬─────────┐
│ column_name │ column_type  │  null   │   key   │ default │  extra  │
│   varchar   │   varchar    │ varchar │ varchar │ varchar │ varchar │
├─────────────┼──────────────┼─────────┼─────────┼─────────┼─────────┤
│ DateCreated │ TIMESTAMP_NS │ YES     │ NULL    │ NULL    │ NULL    │
│ DateUpdated │ TIMESTAMP_NS │ YES     │ NULL    │ NULL    │ NULL    │
│ Field1      │ VARCHAR      │ YES     │ NULL    │ NULL    │ NULL    │
│ Field2      │ VARCHAR      │ YES     │ NULL    │ NULL    │ NULL    │
│ Field3      │ VARCHAR      │ YES     │ NULL    │ NULL    │ NULL    │
│ Data1       │ BIGINT       │ YES     │ NULL    │ NULL    │ NULL    │
│ Data2       │ VARCHAR      │ YES     │ NULL    │ NULL    │ NULL    │
│ Data3       │ VARCHAR      │ YES     │ NULL    │ NULL    │ NULL    │
│ Data4       │ VARCHAR      │ YES     │ NULL    │ NULL    │ NULL    │
│ Data5       │ VARCHAR      │ YES     │ NULL    │ NULL    │ NULL    │
│ Data6       │ BIGINT       │ YES     │ NULL    │ NULL    │ NULL    │
│ Data7       │ BIGINT       │ YES     │ NULL    │ NULL    │ NULL    │
│ Data8       │ VARCHAR      │ YES     │ NULL    │ NULL    │ NULL    │
│ Data9       │ BIGINT       │ YES     │ NULL    │ NULL    │ NULL    │
│ Data10      │ BIGINT       │ YES     │ NULL    │ NULL    │ NULL    │
├─────────────┴──────────────┴─────────┴─────────┴─────────┴─────────┤
│ 15 rows                                                  6 columns │
└────────────────────────────────────────────────────────────────────┘
┌───────────────┬──────────────┬─────────────┬────────────┬───────────────────────┬───────────┬────────────┬───────────┬─────────┬───────────────────────┬───────────┬────────────┬─────────────┬───────────────────────────────────────────────────────────┐
│ database_name │ database_oid │ schema_name │ schema_oid │      index_name       │ index_oid │ table_name │ table_oid │ comment │         tags          │ is_unique │ is_primary │ expressions │                            sql                            │
│    varchar    │    int64     │   varchar   │   int64    │        varchar        │   int64   │  varchar   │   int64   │ varchar │ map(varchar, varchar) │  boolean  │  boolean   │   varchar   │                          varchar                          │
├───────────────┼──────────────┼─────────────┼────────────┼───────────────────────┼───────────┼────────────┼───────────┼─────────┼───────────────────────┼───────────┼────────────┼─────────────┼───────────────────────────────────────────────────────────┤
│ duckdb_test   │          570 │ main        │        572 │ idx_test_table_field1 │       581 │ test_table │       575 │ NULL    │ {}                    │ false     │ false      │ [Field1]    │ CREATE INDEX idx_test_table_field1 ON test_table(Field1); │
│ duckdb_test   │          570 │ main        │        572 │ idx_test_table_field2 │       591 │ test_table │       575 │ NULL    │ {}                    │ false     │ false      │ [Field2]    │ CREATE INDEX idx_test_table_field2 ON test_table(Field2); │
└───────────────┴──────────────┴─────────────┴────────────┴───────────────────────┴───────────┴────────────┴───────────┴─────────┴───────────────────────┴───────────┴────────────┴─────────────┴───────────────────────────────────────────────────────────┘
┌────────────────────────────┬──────────────┐
│ 'test_table record count:' │ count_star() │
│          varchar           │    int64     │
├────────────────────────────┼──────────────┤
│ test_table record count:   │   7500000    │
└────────────────────────────┴──────────────┘

=== SQLITE DATABASE SCHEMA AND RECORD COUNTS ===
CREATE TABLE test_table(DateCreated VARCHAR, DateUpdated VARCHAR, Field1 VARCHAR, Field2 VARCHAR, Field3 VARCHAR, Data1 BIGINT, Data2 VARCHAR, Data3 VARCHAR, Data4 VARCHAR, Data5 VARCHAR, Data6 BIGINT, Data7 BIGINT, Data8 VARCHAR, Data9 BIGINT, Data10 BIGINT);
CREATE INDEX idx_test_table_field1 ON test_table(Field1);
CREATE INDEX idx_test_table_field2 ON test_table(Field2);
CREATE TABLE test_table2(DateCreated VARCHAR, DateUpdated VARCHAR, Field1 VARCHAR, Field2 VARCHAR, Field3 VARCHAR, Data1 BIGINT, Data2 VARCHAR, Data3 VARCHAR, Data4 VARCHAR, Data5 VARCHAR, Data6 BIGINT, Data7 BIGINT, Data8 VARCHAR, Data9 BIGINT, Data10 BIGINT);
CREATE INDEX idx_test_table2_composite ON test_table2(Field1, Field2);
test_table record count:|7500000
test_table2 record count:|7500000
```

## Run tests

### Start the Server

The server will start on `http://localhost:8090` with the following endpoints:

- `POST /v1/sql_duckdb` - Execute queries against DuckDB
- `POST /v1/sql_turso` - Execute queries against SQLite (Limbo)

```bash
# Using Make
make run

# Or directly with Cargo
cargo run --release
```

Output:

```bash
..
    Finished `release` profile [optimized] target(s) in 2m 50s
     Running `target/release/sql-query-server`
SQL query server running at http://localhost:8090
SQL endpoint 1: POST http://localhost:8090/v1/sql_duckdb
SQL endpoint 2: POST http://localhost:8090/v1/sql_turso
```

### Test Queries

Execute sample queries to verify the setup:

```bash
make test-queries
```

Output:

```bash
DuckDB Test Query

curl -X POST -H "Content-Type: application/text" -d "SELECT Field3 FROM test_table WHERE Field1 = 'F1931ea3af-73c8-4ad3-990e-0f2add780352' AND Field2='F2429cc655-8c98-489b-b826-d97787bea6c0';" http://localhost:8090/v1/sql_duckdb
{"result":"[RecordBatch { schema: Schema { fields: [Field { name: \"Field3\", data_type: Utf8, nullable: true, dict_id: 0, dict_is_ordered: false, metadata: {} }], metadata: {} }, columns: [StringArray\n[\n  \"F3fc2b5231-6a17-4c88-b0ab-f2f3fc8b5ce5\",\n]], row_count: 1 }]"}
Turso Test Query (table1)

curl -X POST -H "Content-Type: application/text" -d "SELECT Field3 FROM test_table WHERE Field1 = 'F1931ea3af-73c8-4ad3-990e-0f2add780352' AND Field2='F2429cc655-8c98-489b-b826-d97787bea6c0';" http://localhost:8090/v1/sql_turso
{"result":"[Row { values: [Text(Text { value: [70, 51, 102, 99, 50, 98, 53, 50, 51, 49, 45, 54, 97, 49, 55, 45, 52, 99, 56, 56, 45, 98, 48, 97, 98, 45, 102, 50, 102, 51, 102, 99, 56, 98, 53, 99, 101, 53], subtype: Text })] }]"}
Turso Test Query (table2)

curl -X POST -H "Content-Type: application/text" -d "SELECT Field3 FROM test_table2 WHERE Field1 = 'F1931ea3af-73c8-4ad3-990e-0f2add780352' AND Field2='F2429cc655-8c98-489b-b826-d97787bea6c0';" http://localhost:8090/v1/sql_turso
{"result":"[Row { values: [Text(Text { value: [70, 51, 102, 99, 50, 98, 53, 50, 51, 49, 45, 54, 97, 49, 55, 45, 52, 99, 56, 56, 45, 98, 48, 97, 98, 45, 102, 50, 102, 51, 102, 99, 56, 98, 53, 99, 101, 53], subtype: Text })] }]"}%   
```

### Performance Benchmarking

Run load tests using OHA:

```bash
make test
```

This executes:

1. **DuckDB test**: 10 concurrent connections/queries for 10 seconds
2. **SQLite test (separate indexes)**: Same load against `test_table`
3. **SQLite test (composite index)**: Same load against `test_table2`

#### Output (DuckDB):

```bash
oha --method POST "http://127.0.0.1:8090/v1/sql_duckdb" -D test_query-01.sql -z 10s -c 10
Summary:
  Success rate:	100.00%
  Total:	10.0030 secs
  Slowest:	0.7624 secs
  Fastest:	0.0915 secs
  Average:	0.2780 secs
  Requests/sec:	36.2892

  Total data:	94.46 KiB
  Size/request:	274 B
  Size/sec:	9.44 KiB

Response time histogram:
  0.091 [1]   |
  0.159 [90]  |■■■■■■■■■■■■■■■■■■■
  0.226 [147] |■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■
  0.293 [26]  |■■■■■
  0.360 [8]   |■
  0.427 [6]   |■
  0.494 [1]   |
  0.561 [13]  |■■
  0.628 [18]  |■■■
  0.695 [39]  |■■■■■■■■
  0.762 [4]   |

Response time distribution:
  10.00% in 0.1406 secs
  25.00% in 0.1579 secs
  50.00% in 0.1875 secs
  75.00% in 0.2968 secs
  90.00% in 0.6447 secs
  95.00% in 0.6598 secs
  99.00% in 0.7008 secs
  99.90% in 0.7624 secs
  99.99% in 0.7624 secs


Details (average, fastest, slowest):
  DNS+dialup:	0.0004 secs, 0.0002 secs, 0.0005 secs
  DNS-lookup:	0.0000 secs, 0.0000 secs, 0.0001 secs

Status code distribution:
  [200] 353 responses

Error distribution:
  [10] aborted due to deadline
```

#### Output (Turso / separate indexes):

```bash
oha --method POST "http://127.0.0.1:8090/v1/sql_turso" -D test_query-01.sql -z 10s -c 10
Summary:
  Success rate:	100.00%
  Total:	10.0007 secs
  Slowest:	0.0058 secs
  Fastest:	0.0000 secs
  Average:	0.0001 secs
  Requests/sec:	121502.8025

  Total data:	263.05 MiB
  Size/request:	227 B
  Size/sec:	26.30 MiB

Response time histogram:
  0.000 [1]       |
  0.001 [1214901] |■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■
  0.001 [62]      |
  0.002 [14]      |
  0.002 [8]       |
  0.003 [20]      |
  0.003 [20]      |
  0.004 [20]      |
  0.005 [30]      |
  0.005 [10]      |
  0.006 [30]      |

Response time distribution:
  10.00% in 0.0001 secs
  25.00% in 0.0001 secs
  50.00% in 0.0001 secs
  75.00% in 0.0001 secs
  90.00% in 0.0001 secs
  95.00% in 0.0001 secs
  99.00% in 0.0001 secs
  99.90% in 0.0002 secs
  99.99% in 0.0024 secs


Details (average, fastest, slowest):
  DNS+dialup:	0.0005 secs, 0.0003 secs, 0.0007 secs
  DNS-lookup:	0.0000 secs, 0.0000 secs, 0.0001 secs

Status code distribution:
  [200] 1215116 responses

Error distribution:
  [3] aborted due to deadline
```

#### Output (Turso / composite index):

```bash
oha --method POST "http://127.0.0.1:8090/v1/sql_turso" -D test_query-01_tbl2.sql -z 10s -c 10
Summary:
  Success rate:	100.00%
  Total:	10.0011 secs
  Slowest:	0.0060 secs
  Fastest:	0.0000 secs
  Average:	0.0001 secs
  Requests/sec:	122605.5252

  Total data:	265.45 MiB
  Size/request:	227 B
  Size/sec:	26.54 MiB

Response time histogram:
  0.000 [1]       |
  0.001 [1226051] |■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■
  0.001 [10]      |
  0.002 [0]       |
  0.002 [0]       |
  0.003 [20]      |
  0.004 [0]       |
  0.004 [10]      |
  0.005 [23]      |
  0.005 [47]      |
  0.006 [20]      |

Response time distribution:
  10.00% in 0.0001 secs
  25.00% in 0.0001 secs
  50.00% in 0.0001 secs
  75.00% in 0.0001 secs
  90.00% in 0.0001 secs
  95.00% in 0.0001 secs
  99.00% in 0.0001 secs
  99.90% in 0.0002 secs
  99.99% in 0.0012 secs


Details (average, fastest, slowest):
  DNS+dialup:	0.0003 secs, 0.0002 secs, 0.0004 secs
  DNS-lookup:	0.0000 secs, 0.0000 secs, 0.0001 secs

Status code distribution:
  [200] 1226182 responses

Error distribution:
  [4] aborted due to deadline
```

## Notes

DuckDB (v1.3.1) does not appear to use indexes when scanning; it uses `SEQ_SCAN`.

```bash
D SELECT Field3 FROM test_table WHERE Field1 = 'F1931ea3af-73c8-4ad3-990e-0f2add780352' AND Field2='F2429cc655-8c98-489b-b826-d97787bea6c0';
┌────────────────────────────────────────┐
│                 Field3                 │
│                varchar                 │
├────────────────────────────────────────┤
│ F3fc2b5231-6a17-4c88-b0ab-f2f3fc8b5ce5 │
└────────────────────────────────────────┘
Run Time (s): real 0.054 user 0.564931 sys 0.003300
```

```bash
D explain SELECT Field3 FROM test_table WHERE Field1 = 'F1931ea3af-73c8-4ad3-990e-0f2add780352' AND Field2='F2429cc655-8c98-489b-b826-d97787bea6c0';

┌─────────────────────────────┐
│┌───────────────────────────┐│
││       Physical Plan       ││
│└───────────────────────────┘│
└─────────────────────────────┘
┌───────────────────────────┐
│         SEQ_SCAN          │
│    ────────────────────   │
│     Table: test_table     │
│   Type: Sequential Scan   │
│    Projections: Field3    │
│                           │
│          Filters:         │
│  Field1='F1931ea3af-73c8  │
│  -4ad3-990e-0f2add780352' │
│  Field2='F2429cc655-8c98  │
│  -489b-b826-d97787bea6c0' │
│                           │
│          ~2 Rows          │
└───────────────────────────┘
Run Time (s): real 0.002 user 0.000496 sys 0.000212
```

Indexes exist:

```bash
D SELECT * FROM duckdb_indexes WHERE table_name = 'test_table';
┌───────────────┬──────────────┬─────────────┬────────────┬──────────────────────┬───────────┬────────────┬───────────┬─────────┬──────────────────────┬───────────┬────────────┬─────────────┬──────────────────────────────────────────────────────┐
│ database_name │ database_oid │ schema_name │ schema_oid │      index_name      │ index_oid │ table_name │ table_oid │ comment │         tags         │ is_unique │ is_primary │ expressions │                         sql                          │
│    varchar    │    int64     │   varchar   │   int64    │       varchar        │   int64   │  varchar   │   int64   │ varchar │ map(varchar, varch…  │  boolean  │  boolean   │   varchar   │                       varchar                        │
├───────────────┼──────────────┼─────────────┼────────────┼──────────────────────┼───────────┼────────────┼───────────┼─────────┼──────────────────────┼───────────┼────────────┼─────────────┼──────────────────────────────────────────────────────┤
│ duckdb_test   │          546 │ main        │        548 │ idx_test_table_fie…  │       557 │ test_table │       551 │ NULL    │ {}                   │ false     │ false      │ [Field1]    │ CREATE INDEX idx_test_table_field1 ON test_table(F…  │
│ duckdb_test   │          546 │ main        │        548 │ idx_test_table_fie…  │       567 │ test_table │       551 │ NULL    │ {}                   │ false     │ false      │ [Field2]    │ CREATE INDEX idx_test_table_field2 ON test_table(F…  │
└───────────────┴──────────────┴─────────────┴────────────┴──────────────────────┴───────────┴────────────┴───────────┴─────────┴──────────────────────┴───────────┴────────────┴─────────────┴──────────────────────────────────────────────────────┘
Run Time (s): real 0.002 user 0.001433 sys 0.000437
```