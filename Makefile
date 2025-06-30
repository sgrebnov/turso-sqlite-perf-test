run:
	cargo run --release

test-queries:
	@echo "\nDuckDB Test Query\n"
	curl -X POST -H "Content-Type: application/text" -d "SELECT Field3 FROM test_table WHERE Field1 = 'F1931ea3af-73c8-4ad3-990e-0f2add780352' AND Field2='F2429cc655-8c98-489b-b826-d97787bea6c0';" http://localhost:8090/v1/sql_duckdb
	@echo "\nTurso Test Query (table1)\n"
	curl -X POST -H "Content-Type: application/text" -d "SELECT Field3 FROM test_table WHERE Field1 = 'F1931ea3af-73c8-4ad3-990e-0f2add780352' AND Field2='F2429cc655-8c98-489b-b826-d97787bea6c0';" http://localhost:8090/v1/sql_turso
	@echo "\nTurso Test Query (table2)\n"
	curl -X POST -H "Content-Type: application/text" -d "SELECT Field3 FROM test_table2 WHERE Field1 = 'F1931ea3af-73c8-4ad3-990e-0f2add780352' AND Field2='F2429cc655-8c98-489b-b826-d97787bea6c0';" http://localhost:8090/v1/sql_turso

test:
	oha --method POST "http://127.0.0.1:8090/v1/sql_duckdb" -D test_query-01.sql -z 10s -c 10
	oha --method POST "http://127.0.0.1:8090/v1/sql_turso" -D test_query-01.sql -z 10s -c 10
	oha --method POST "http://127.0.0.1:8090/v1/sql_turso" -D test_query-01_tbl2.sql -z 10s -c 10
