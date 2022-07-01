// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![warn(clippy::all)]
//! Test SQL syntax specific to MySQL. The parser based on the generic dialect
//! is also tested (on the inputs it can handle).

#[macro_use]
mod test_utils;

use test_utils::*;

use sqlparser::ast::*;
use sqlparser::dialect::{GenericDialect, MySqlDialect};
use sqlparser::tokenizer::Token;

#[test]
fn parse_identifiers() {
    mysql().verified_stmt("SELECT $a$, àà");
}

#[test]
fn parse_show_columns() {
    let table_name = ObjectName(vec![Ident::new("mytable")]);
    assert_eq!(
        mysql_and_generic().verified_stmt("SHOW COLUMNS FROM mytable"),
        Statement::ShowColumns {
            extended: false,
            full: false,
            table_name: table_name.clone(),
            filter: None,
        }
    );
    assert_eq!(
        mysql_and_generic().verified_stmt("SHOW COLUMNS FROM mydb.mytable"),
        Statement::ShowColumns {
            extended: false,
            full: false,
            table_name: ObjectName(vec![Ident::new("mydb"), Ident::new("mytable")]),
            filter: None,
        }
    );
    assert_eq!(
        mysql_and_generic().verified_stmt("SHOW EXTENDED COLUMNS FROM mytable"),
        Statement::ShowColumns {
            extended: true,
            full: false,
            table_name: table_name.clone(),
            filter: None,
        }
    );
    assert_eq!(
        mysql_and_generic().verified_stmt("SHOW FULL COLUMNS FROM mytable"),
        Statement::ShowColumns {
            extended: false,
            full: true,
            table_name: table_name.clone(),
            filter: None,
        }
    );
    assert_eq!(
        mysql_and_generic().verified_stmt("SHOW COLUMNS FROM mytable LIKE 'pattern'"),
        Statement::ShowColumns {
            extended: false,
            full: false,
            table_name: table_name.clone(),
            filter: Some(ShowStatementFilter::Like("pattern".into())),
        }
    );
    assert_eq!(
        mysql_and_generic().verified_stmt("SHOW COLUMNS FROM mytable WHERE 1 = 2"),
        Statement::ShowColumns {
            extended: false,
            full: false,
            table_name,
            filter: Some(ShowStatementFilter::Where(
                mysql_and_generic().verified_expr("1 = 2")
            )),
        }
    );
    mysql_and_generic()
        .one_statement_parses_to("SHOW FIELDS FROM mytable", "SHOW COLUMNS FROM mytable");
    mysql_and_generic()
        .one_statement_parses_to("SHOW COLUMNS IN mytable", "SHOW COLUMNS FROM mytable");
    mysql_and_generic()
        .one_statement_parses_to("SHOW FIELDS IN mytable", "SHOW COLUMNS FROM mytable");

    // unhandled things are truly unhandled
    match mysql_and_generic().parse_sql_statements("SHOW COLUMNS FROM mytable FROM mydb") {
        Err(_) => {}
        Ok(val) => panic!("unexpected successful parse: {:?}", val),
    }
}

#[test]
fn parse_show_create() {
    let obj_name = ObjectName(vec![Ident::new("myident")]);

    for obj_type in &vec![
        ShowCreateObject::Table,
        ShowCreateObject::Trigger,
        ShowCreateObject::Event,
        ShowCreateObject::Function,
        ShowCreateObject::Procedure,
    ] {
        assert_eq!(
            mysql_and_generic().verified_stmt(format!("SHOW CREATE {} myident", obj_type).as_str()),
            Statement::ShowCreate {
                obj_type: obj_type.clone(),
                obj_name: obj_name.clone(),
            }
        );
    }
}

#[test]
fn parse_create_table_auto_increment() {
    let sql = "CREATE TABLE foo (bar INT PRIMARY KEY AUTO_INCREMENT)";
    match mysql().verified_stmt(sql) {
        Statement::CreateTable { name, columns, .. } => {
            assert_eq!(name.to_string(), "foo");
            assert_eq!(
                vec![ColumnDef {
                    name: Ident::new("bar"),
                    data_type: DataType::Int(None),
                    collation: None,
                    options: vec![
                        ColumnOptionDef {
                            name: None,
                            option: ColumnOption::Unique { is_primary: true },
                        },
                        ColumnOptionDef {
                            name: None,
                            option: ColumnOption::DialectSpecific(vec![Token::make_keyword(
                                "AUTO_INCREMENT"
                            )]),
                        },
                    ],
                }],
                columns
            );
        }
        _ => unreachable!(),
    }
}

#[test]
fn parse_create_table_with_column_comment() {
    let sql = "CREATE TABLE foo (bar INT COMMENT 'bar')";
    match mysql().verified_stmt(sql) {
        Statement::CreateTable { name, columns, .. } => {
            assert_eq!(name.to_string(), "foo");
            assert_eq!(
                vec![ColumnDef {
                    name: Ident::new("bar"),
                    data_type: DataType::Int(None),
                    collation: None,
                    options: vec![ColumnOptionDef {
                        name: None,
                        option: ColumnOption::DialectSpecific(vec![
                            Token::make_keyword("COMMENT"),
                            Token::make_word("bar", Some('\''))
                        ]),
                    },],
                }],
                columns
            );
        }
        _ => unreachable!(),
    }
}

#[test]
fn parse_quote_identifiers() {
    let sql = "CREATE TABLE `PRIMARY` (`BEGIN` INT PRIMARY KEY)";
    match mysql().verified_stmt(sql) {
        Statement::CreateTable { name, columns, .. } => {
            assert_eq!(name.to_string(), "`PRIMARY`");
            assert_eq!(
                vec![ColumnDef {
                    name: Ident::with_quote('`', "BEGIN"),
                    data_type: DataType::Int(None),
                    collation: None,
                    options: vec![ColumnOptionDef {
                        name: None,
                        option: ColumnOption::Unique { is_primary: true },
                    }],
                }],
                columns
            );
        }
        _ => unreachable!(),
    }
}

#[test]
fn parse_create_table_with_minimum_display_width() {
    let sql = "CREATE TABLE foo (bar_tinyint TINYINT(3), bar_smallint SMALLINT(5), bar_int INT(11), bar_bigint BIGINT(20))";
    match mysql().verified_stmt(sql) {
        Statement::CreateTable { name, columns, .. } => {
            assert_eq!(name.to_string(), "foo");
            assert_eq!(
                vec![
                    ColumnDef {
                        name: Ident::new("bar_tinyint"),
                        data_type: DataType::TinyInt(Some(3)),
                        collation: None,
                        options: vec![],
                    },
                    ColumnDef {
                        name: Ident::new("bar_smallint"),
                        data_type: DataType::SmallInt(Some(5)),
                        collation: None,
                        options: vec![],
                    },
                    ColumnDef {
                        name: Ident::new("bar_int"),
                        data_type: DataType::Int(Some(11)),
                        collation: None,
                        options: vec![],
                    },
                    ColumnDef {
                        name: Ident::new("bar_bigint"),
                        data_type: DataType::BigInt(Some(20)),
                        collation: None,
                        options: vec![],
                    }
                ],
                columns
            );
        }
        _ => unreachable!(),
    }
}

#[test]
fn parse_create_table_unsigned() {
    let sql = "CREATE TABLE foo (bar_tinyint TINYINT(3) UNSIGNED, bar_smallint SMALLINT(5) UNSIGNED, bar_int INT(11) UNSIGNED, bar_bigint BIGINT(20) UNSIGNED)";
    match mysql().verified_stmt(sql) {
        Statement::CreateTable { name, columns, .. } => {
            assert_eq!(name.to_string(), "foo");
            assert_eq!(
                vec![
                    ColumnDef {
                        name: Ident::new("bar_tinyint"),
                        data_type: DataType::UnsignedTinyInt(Some(3)),
                        collation: None,
                        options: vec![],
                    },
                    ColumnDef {
                        name: Ident::new("bar_smallint"),
                        data_type: DataType::UnsignedSmallInt(Some(5)),
                        collation: None,
                        options: vec![],
                    },
                    ColumnDef {
                        name: Ident::new("bar_int"),
                        data_type: DataType::UnsignedInt(Some(11)),
                        collation: None,
                        options: vec![],
                    },
                    ColumnDef {
                        name: Ident::new("bar_bigint"),
                        data_type: DataType::UnsignedBigInt(Some(20)),
                        collation: None,
                        options: vec![],
                    },
                ],
                columns
            );
        }
        _ => unreachable!(),
    }
}

#[test]
#[cfg(not(feature = "bigdecimal"))]
fn parse_simple_insert() {
    let sql = r"INSERT INTO tasks (title, priority) VALUES ('Test Some Inserts', 1), ('Test Entry 2', 2), ('Test Entry 3', 3)";

    match mysql().verified_stmt(sql) {
        Statement::Insert {
            table_name,
            columns,
            source,
            on,
            ..
        } => {
            assert_eq!(ObjectName(vec![Ident::new("tasks")]), table_name);
            assert_eq!(vec![Ident::new("title"), Ident::new("priority")], columns);
            assert!(on.is_none());
            assert_eq!(
                Some(Box::new(Query {
                    with: None,
                    body: SetExpr::Values(Values(vec![
                        vec![
                            Expr::Value(Value::SingleQuotedString("Test Some Inserts".to_string())),
                            Expr::Value(Value::Number("1".to_string(), false))
                        ],
                        vec![
                            Expr::Value(Value::SingleQuotedString("Test Entry 2".to_string())),
                            Expr::Value(Value::Number("2".to_string(), false))
                        ],
                        vec![
                            Expr::Value(Value::SingleQuotedString("Test Entry 3".to_string())),
                            Expr::Value(Value::Number("3".to_string(), false))
                        ]
                    ])),
                    order_by: vec![],
                    limit: None,
                    offset: None,
                    fetch: None,
                    format: None,
                })),
                source,
            );
        }
        _ => unreachable!(),
    }

    let sql = r"INSERT INTO array_test (nums) VALUES ([]), ([1, 2, 3]), ([4, 5, 6])";
    match mysql().verified_stmt(sql) {
        Statement::Insert {
            table_name,
            columns,
            source,
            on,
            ..
        } => {
            assert_eq!(ObjectName(vec![Ident::new("array_test")]), table_name);
            assert_eq!(vec![Ident::new("nums")], columns);
            assert!(on.is_none());
            assert_eq!(
                Some(Box::new(Query {
                    with: None,
                    body: SetExpr::Values(Values(vec![
                        vec![Expr::Array(vec![]),],
                        vec![Expr::Array(vec![
                            Expr::Value(number("1")),
                            Expr::Value(number("2")),
                            Expr::Value(number("3"))
                        ]),],
                        vec![Expr::Array(vec![
                            Expr::Value(number("4")),
                            Expr::Value(number("5")),
                            Expr::Value(number("6"))
                        ]),],
                    ])),
                    order_by: vec![],
                    limit: None,
                    offset: None,
                    fetch: None,
                    format: None,
                })),
                source,
            );
        }
        _ => unreachable!(),
    }
}

#[test]
fn parse_double_quoted() {
    let sql_double_quoted =
        r#"INSERT INTO tasks (title, priority) VALUES ("Test S\"o\"m'e' In'se\"rt`s\"", 1)"#;
    let mut double_statements = mysql().parse_sql_statements(sql_double_quoted).unwrap();
    assert_eq!(double_statements.len(), 1);
    let double_statement = double_statements.pop().unwrap();
    match double_statement {
        Statement::Insert { source, .. } => {
            assert_eq!(
                source.unwrap().body.to_string(),
                "VALUES (\"Test S\"o\"m'e' In'se\"rt`s\"\", 1)"
            )
        }
        _ => unreachable!(),
    }

    let select_sql = r#"SELECT obj["c"][0] FROM t4"#;
    mysql().verified_stmt(select_sql);

    let sql = r#"SELECT * FROM customers WHERE name LIKE "%a" IS NULL"#;
    let select = mysql().verified_only_select(sql);
    assert_eq!(
        Expr::IsNull(Box::new(Expr::BinaryOp {
            left: Box::new(Expr::Identifier(Ident::new("name"))),
            op: BinaryOperator::Like,
            right: Box::new(Expr::Value(Value::DoubleQuotedString("%a".to_string()))),
        })),
        select.selection.unwrap()
    );
}

#[test]
fn parse_update_with_joins() {
    let sql = "UPDATE orders AS o JOIN customers AS c ON o.customer_id = c.id SET o.completed = true WHERE c.firstname = 'Peter'";
    match mysql().verified_stmt(sql) {
        Statement::Update {
            table,
            assignments,
            selection,
        } => {
            assert_eq!(
                TableWithJoins {
                    relation: TableFactor::Table {
                        name: ObjectName(vec![Ident::new("orders")]),
                        alias: Some(TableAlias {
                            name: Ident::new("o"),
                            columns: vec![]
                        }),
                        args: vec![],
                        with_hints: vec![],
                        instant: None,
                    },
                    joins: vec![Join {
                        relation: TableFactor::Table {
                            name: ObjectName(vec![Ident::new("customers")]),
                            alias: Some(TableAlias {
                                name: Ident::new("c"),
                                columns: vec![]
                            }),
                            args: vec![],
                            with_hints: vec![],
                            instant: None,
                        },
                        join_operator: JoinOperator::Inner(JoinConstraint::On(Expr::BinaryOp {
                            left: Box::new(Expr::CompoundIdentifier(vec![
                                Ident::new("o"),
                                Ident::new("customer_id")
                            ])),
                            op: BinaryOperator::Eq,
                            right: Box::new(Expr::CompoundIdentifier(vec![
                                Ident::new("c"),
                                Ident::new("id")
                            ]))
                        })),
                    }]
                },
                table
            );
            assert_eq!(
                vec![Assignment {
                    id: vec![Ident::new("o"), Ident::new("completed")],
                    value: Expr::Value(Value::Boolean(true))
                }],
                assignments
            );
            assert_eq!(
                Some(Expr::BinaryOp {
                    left: Box::new(Expr::CompoundIdentifier(vec![
                        Ident::new("c"),
                        Ident::new("firstname")
                    ])),
                    op: BinaryOperator::Eq,
                    right: Box::new(Expr::Value(Value::SingleQuotedString("Peter".to_string())))
                }),
                selection
            );
        }
        _ => unreachable!(),
    }
}

#[test]
fn parse_alter_table_change_column() {
    let expected_name = ObjectName(vec![Ident::new("orders")]);
    let expected_operation = AlterTableOperation::ChangeColumn {
        old_name: Ident::new("description"),
        new_name: Ident::new("desc"),
        data_type: DataType::Text,
        options: vec![ColumnOption::NotNull],
    };

    let sql1 = "ALTER TABLE orders CHANGE COLUMN description desc TEXT NOT NULL";
    match mysql().verified_stmt(sql1) {
        Statement::AlterTable { name, operation } => {
            assert_eq!(expected_name, name);
            assert_eq!(expected_operation, operation);
        }
        _ => unreachable!(),
    }

    let sql2 = "ALTER TABLE orders CHANGE description desc TEXT NOT NULL";
    match mysql().one_statement_parses_to(sql2, sql1) {
        Statement::AlterTable { name, operation } => {
            assert_eq!(expected_name, name);
            assert_eq!(expected_operation, operation);
        }
        _ => unreachable!(),
    }
}

#[test]
#[cfg(not(feature = "bigdecimal"))]
fn parse_substring_in_select() {
    let sql = "SELECT DISTINCT SUBSTRING(description, 0, 1) FROM test";
    match mysql().one_statement_parses_to(
        sql,
        "SELECT DISTINCT SUBSTRING(description FROM 0 FOR 1) FROM test",
    ) {
        Statement::Query(query) => {
            assert_eq!(
                Box::new(Query {
                    with: None,
                    body: SetExpr::Select(Box::new(Select {
                        distinct: true,
                        top: None,
                        projection: vec![SelectItem::UnnamedExpr(Expr::Substring {
                            expr: Box::new(Expr::Identifier(Ident {
                                value: "description".to_string(),
                                quote_style: None
                            })),
                            substring_from: Some(Box::new(Expr::Value(Value::Number(
                                "0".to_string(),
                                false
                            )))),
                            substring_for: Some(Box::new(Expr::Value(Value::Number(
                                "1".to_string(),
                                false
                            ))))
                        })],
                        from: vec![TableWithJoins {
                            relation: TableFactor::Table {
                                name: ObjectName(vec![Ident {
                                    value: "test".to_string(),
                                    quote_style: None
                                }]),
                                alias: None,
                                args: vec![],
                                with_hints: vec![],
                                instant: None,
                            },
                            joins: vec![]
                        }],
                        lateral_views: vec![],
                        selection: None,
                        group_by: vec![],
                        cluster_by: vec![],
                        distribute_by: vec![],
                        sort_by: vec![],
                        having: None,
                    })),
                    order_by: vec![],
                    limit: None,
                    offset: None,
                    fetch: None,
                    format: None,
                }),
                query
            );
        }
        _ => unreachable!(),
    }
}

fn mysql() -> TestedDialects {
    TestedDialects {
        dialects: vec![Box::new(MySqlDialect {})],
    }
}

fn mysql_and_generic() -> TestedDialects {
    TestedDialects {
        dialects: vec![Box::new(MySqlDialect {}), Box::new(GenericDialect {})],
    }
}
