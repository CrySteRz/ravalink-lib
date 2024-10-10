use crate::RavalinkConfig;
use ravalink_interconnect::protocol::Message;
use nanoid::nanoid;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use std::time::Duration;

fn configure_kafka_ssl(mut kafka_config: ClientConfig, config: &RavalinkConfig) -> ClientConfig {
    if config.ssl.is_some() {
        let ssl = config.ssl.clone().unwrap();
        kafka_config
            .set("security.protocol", "ssl")
            .set("ssl.ca.location", ssl.ssl_ca)
            .set("ssl.certificate.location", ssl.ssl_cert)
            .set("ssl.key.location", ssl.ssl_key);
    } else if config.sasl.is_some() {
        let sasl = config.sasl.clone().unwrap();
        kafka_config
            .set("security.protocol", "SASL_PLAINTEXT")
            .set("sasl.mechanisms", "PLAIN")
            .set("sasl.username", sasl.kafka_username)
            .set("sasl.password", sasl.kafka_password);
    }
    kafka_config
}

pub fn initialize_producer(broker: &str, config: &RavalinkConfig) -> FutureProducer {
    let mut kafka_config = ClientConfig::new().set("bootstrap.servers", broker).clone();
    kafka_config = configure_kafka_ssl(kafka_config, config);
    let producer: FutureProducer = kafka_config.create().expect("Failed to create Producer");
    
    producer
}

pub async fn initialize_client(brokers: &String, config: &RavalinkConfig) -> StreamConsumer {
    let mut kafka_config = ClientConfig::new()
        .set("group.id", nanoid!())
        .set("bootstrap.servers", brokers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set("security.protocol", "ssl")
        .clone();

    kafka_config = configure_kafka_ssl(kafka_config, config);

    let consumer: StreamConsumer = kafka_config.create().expect("Failed to create Consumer");

    consumer
        .subscribe(&[&config.kafka_topic])
        .expect("Can't subscribe to specified topic");

    consumer
}

pub async fn send_message(message: &Message, topic: &str, producer: &mut FutureProducer) {
    let data = serde_json::to_string(message).unwrap();
    let record: FutureRecord<String, String> = FutureRecord::to(topic).payload(&data);
    producer.send(record, Duration::from_secs(1)).await.unwrap();
}