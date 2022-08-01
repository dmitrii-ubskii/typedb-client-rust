/*
 * Copyright (C) 2021 Vaticle
 *
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */

// #![feature(if_let_guard)] // only available on nightly Rust builds

mod concept_models {
    use futures::StreamExt;
    use std::time::Instant;
    use typedb_client::{concept, concept2, session, Session, transaction, TypeDBClient};
    use typedb_client::concept::{Attribute, Concept, Thing};
    // use typedb_client::concept2::{Concept, Entity, LongAttribute, StringAttribute, Thing, ThingType, Type};
    // use typedb_client::concept::Attribute;
    use typedb_client::session::Type::{Data, Schema};
    use typedb_client::transaction::Transaction;
    use typedb_client::transaction::Type::{Read, Write};

    const GRAKN: &str = "grakn";

    async fn new_typedb_client() -> TypeDBClient {
        TypeDBClient::new("0.0.0.0", 1729).await.unwrap_or_else(|err| panic!("An error occurred connecting to TypeDB Server: {}", err))
    }

    async fn create_db_grakn(client: &TypeDBClient) {
        match client.databases.contains(GRAKN).await {
            Ok(true) => {
                let grakn = client.databases.get(GRAKN).await.unwrap_or_else(|err| panic!("An error occurred getting database '{}': {}", GRAKN, err));
                grakn.delete().await.unwrap_or_else(|err| panic!("An error occurred deleting database '{}': {}", GRAKN, err))
            }
            Err(err) => { panic!("An error occurred checking if the database '{}' exists: {}", GRAKN, err) }
            _ => {}
        }
        client.databases.create(GRAKN).await.unwrap_or_else(|err| panic!("An error occurred creating database '{}': {}", GRAKN, err));
    }

    async fn new_session(client: &TypeDBClient, session_type: session::Type) -> Session {
        client.session(GRAKN, session_type).await.unwrap_or_else(|err| panic!("An error occurred opening a session: {}", err))
    }

    async fn new_tx(session: &Session, tx_type: transaction::Type) -> Transaction {
        session.transaction(tx_type).await.unwrap_or_else(|err| panic!("An error occurred opening a transaction: {}", err))
    }

    async fn commit_tx(tx: &Transaction) {
        tx.commit().await.unwrap_or_else(|err| panic!("An error occurred committing a transaction: {}", err))
    }

    async fn run_define_query(tx: &Transaction, query: &str) {
        tx.query().define(query).await.unwrap_or_else(|err| panic!("An error occurred running a Define query: {}", err));
    }

    #[allow(unused_must_use)]
    fn run_insert_query(tx: &Transaction, query: &str) {
        tx.query().insert(query);
    }

    #[tokio::test]
    async fn concept_api2() {
        let client = new_typedb_client().await;
        create_db_grakn(&client).await;
        {
            let session = new_session(&client, Schema).await;
            let tx = new_tx(&session, Write).await;
            run_define_query(&tx, "define person sub entity, owns name, owns age; name sub attribute, value string; age sub attribute, value long;").await;
            commit_tx(&tx).await;
        }
        {
            let session = new_session(&client, Data).await;
            let tx = new_tx(&session, Write).await;
            run_insert_query(&tx, "insert $x isa person, has name \"Alice\", has age 18; $y isa person, has name \"Bob\", has age 21;");
            commit_tx(&tx).await;
        }
        let session = new_session(&client, Data).await;
        let tx = new_tx(&session, Read).await;
        let mut answer_stream = tx.query2().match_("match $x isa thing;");
        while let Some(result) = answer_stream.next().await {
            match result {
                Ok(concept_map) => {
                    // naive print
                    // println!("test:concept_api: got answer: {:#?}", concept_map);
                    for concept in concept_map {
                        // trait-object(1) match guard then convert
                        match concept {
                            x if x.is_entity() => { describe_entity2(x.as_entity().unwrap()).await; }
                            x if x.is_attribute() => {
                                let attr = x.as_attribute().unwrap();
                                match attr {
                                    y if y.is_long() => { describe_long_attr2(y.as_long().unwrap()).await; }
                                    y if y.is_string() => { describe_str_attr2(y.as_string().unwrap()).await; }
                                    _ => panic!()
                                 }
                            }
                            _ => { todo!() }
                        }
                    }
                }
                Err(err) => panic!("An error occurred fetching answers of a Match query: {}", err)
            }
        }
    }

    #[tokio::test]
    async fn concept_api() {
        let client = new_typedb_client().await;
        create_db_grakn(&client).await;
        {
            let session = new_session(&client, Schema).await;
            let tx = new_tx(&session, Write).await;
            run_define_query(&tx, "define person sub entity, owns name, owns age; name sub attribute, value string; age sub attribute, value long;").await;
            commit_tx(&tx).await;
        }
        {
            let session = new_session(&client, Data).await;
            let tx = new_tx(&session, Write).await;
            run_insert_query(&tx, "insert $x isa person, has name \"Alice\", has age 18; $y isa person, has name \"Bob\", has age 21;");
            commit_tx(&tx).await;
        }
        let session = new_session(&client, Data).await;
        let tx = new_tx(&session, Read).await;
        let mut answer_stream = tx.query().match_("match $x isa thing;");
        while let Some(result) = answer_stream.next().await {
            match result {
                Ok(concept_map) => {
                    // naive print
                    // println!("test:concept_api: got answer: {:#?}", concept_map);
                    for concept in concept_map {
                        // enum(1) branching: safest approach but ugly!
                        match &concept {
                            Concept::Thing(Thing::Entity(entity)) => { describe_entity(entity).await; }
                            Concept::Thing(Thing::Attribute(attr)) => {
                                match &attr {
                                    Attribute::Long(long_attr) => { describe_long_attr(long_attr).await; }
                                    Attribute::String(str_attr) => { describe_str_attr(str_attr).await; }
                                }
                            }
                            _ => {}
                        }

                        // enum(2) match guard then convert: prettier, but unsafe
                        // match &concept {
                        //     x if x.is_entity() => { describe_entity(x.as_entity().unwrap()).await; }
                        //     x if x.is_attribute() => {
                        //         let attr = x.as_attribute().unwrap();
                        //         match attr {
                        //             y if attr.is_long() => { describe_long_attr(y.as_long().unwrap()).await; }
                        //             y if attr.is_string() => { describe_str_attr(y.as_string().unwrap()).await; }
                        //             _ => panic!()
                        //         }
                        //     }
                        //     _ => panic!()
                        // }

                        // enum(3) try-convert in match guard: prettiest approach and mostly safe, but requires nightly Rust build
                        // match &concept {
                        //     entity if let Ok(entity) = concept.as_entity() => { describe_entity(entity).await; }
                        //     attr if let Ok(attr) = concept.as_attribute() => {
                        //         match attr {
                        //             long_attr if let Ok(long_attr) = attr.as_long() => { describe_long_attr(long_attr).await; }
                        //             str_attr if let Ok(str_attr) = attr.as_string() => { describe_str_attr(str_attr).await; }
                        //             _ => panic!()
                        //         }
                        //     }
                        //     _ => panic!()
                        // }
                    }
                }
                Err(err) => panic!("An error occurred fetching answers of a Match query: {}", err)
            }
        }
    }

    async fn describe_entity(entity: &concept::Entity) {
        println!("answer is an ENTITY of type {}", entity.type_.label.as_str());
    }

    async fn describe_long_attr(long_attr: &concept::LongAttribute) {
        println!("answer is a LONG ATTRIBUTE with value {}", long_attr.value);
    }

    async fn describe_str_attr(str_attr: &concept::StringAttribute) {
        println!("answer is a STRING ATTRIBUTE with value {}", str_attr.value);
    }

    // trait-object
    async fn describe_entity2(entity: Box<dyn concept2::Entity>) {
        // NB: this is what we -want- to write, but we get: "multiple applicable items in scope (help: disambiguate the associated function...)"
        // println!("answer is an ENTITY of type {}", entity.get_type().label.as_str());
        println!("answer is an ENTITY of type {}", concept2::Entity::get_type(&*entity).label().name.as_str());
    }

    async fn describe_long_attr2(long_attr: Box<dyn concept2::LongAttribute>) {
        println!("answer is a LONG ATTRIBUTE with value {}", long_attr.get_value());
    }

    async fn describe_str_attr2(str_attr: Box<dyn concept2::StringAttribute>) {
        println!("answer is a STRING ATTRIBUTE with value {}", str_attr.get_value());
    }

    #[tokio::test]
    async fn concept_api_benchmark() {
        let start_time = Instant::now();
        let mut long_attrs: Vec<concept::LongAttribute> = vec![];
        for _ in 0..1_000_000 {
            long_attrs.push(concept::LongAttribute { iid: "123456789".to_string(), value: 42 })
        }
        let mut sum: i64 = 0;
        for _ in 0..10 {
            for long_attr in long_attrs.iter() {
                sum += long_attr.value
            }
        }
        println!("{} (completed in {}ms)", sum, Instant::now().duration_since(start_time).as_millis())
    }

    #[tokio::test]
    async fn concept_api2_benchmark() {
        let start_time = Instant::now();
        let mut long_attrs: Vec<Box<dyn concept2::LongAttribute>> = vec![];
        for _ in 0..1_000_000 {
            long_attrs.push(Box::new(concept2::LongAttributeImpl { iid: "123456789".to_string(), value: 42 }))
        }
        let mut sum: i64 = 0;
        for _ in 0..10 {
            for long_attr in long_attrs.iter() {
                sum += long_attr.get_value()
            }
        }
        println!("{} (completed in {}ms)", sum, Instant::now().duration_since(start_time).as_millis())
    }
}
