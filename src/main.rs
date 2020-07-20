use std::{env, thread, error::Error, fs::read, time::Duration};
use chrono::{DateTime, Local};
use rppal::i2c::I2c;
use rumqtt::{MqttClient, MqttOptions, QoS, ReconnectOptions};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct SendData {
    timestamp: i64,
    temperature: f32,
}

const ADDRESS_ADT7410: u16 = 0x48;

fn main() -> Result<(), Box<dyn Error>> {
    let client_id = env::var("AWS_IOT_CLIENT_ID").expect("AWS_IOT_CLIENT_ID is undefined.");
    let aws_iot_endpoint = env::var("AWS_IOT_ENDPOINT").expect("AWS_IOT_ENDPOINT is undefined.");
    let ca_path = "AmazonRootCA1.pem";
    let client_cert_path = "certificate.pem.crt";
    let client_key_path = "private.pem.key";

    let mqtt_options = MqttOptions::new(client_id, aws_iot_endpoint, 8883)
        .set_ca(read(ca_path).unwrap())
        .set_client_auth(read(client_cert_path).unwrap(), read(client_key_path).unwrap())
        .set_keep_alive(10)
        .set_reconnect_opts(ReconnectOptions::Always(5));

    let (mut mqtt_client, notifications) = MqttClient::start(mqtt_options).unwrap();
    mqtt_client.subscribe("iot/topic", QoS::AtLeastOnce).unwrap();

    let mut i2c = I2c::with_bus(1).expect("Couldn't start i2c. Is the interface enabled?");
    i2c.set_slave_address(ADDRESS_ADT7410).unwrap();

    let sleep_time = Duration::from_secs(10);
    thread::spawn(move || {
        loop {
            let dt: DateTime<Local> = Local::now();
            let timestamp = dt.timestamp();
            let temp = read_temperature(&i2c);
            let data = SendData { timestamp: timestamp, temperature: temp };
            let payload = serde_json::to_string(&data).unwrap();
            thread::sleep(sleep_time);
            mqtt_client.publish("iot/topic", QoS::AtLeastOnce, false, payload).unwrap();
        }
    });

    for notification in notifications {
        println!("{:?}", notification)
    }

    Ok(())
}

fn read_temperature(i2c: &I2c) -> f32 {
    let word = i2c.smbus_read_word(0x00).unwrap();
    let data = ((word & 0xff00)>>8 | (word & 0xff) << 8) >> 3;

    if data & 0x1000 == 0 {
      data as f32 * 0.0625
    } else {
      ((!data & 0x1fff) + 1) as f32 * -0.0625
    }
}
