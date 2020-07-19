use std::error::Error;
use std::thread;
use std::time::Duration;

use rppal::system::DeviceInfo;
use rppal::i2c::I2c;

const ADDRESS_ADT7410: u16 = 0x48;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Blinking an LED on a {}.", DeviceInfo::new()?.model());

    let mut i2c = I2c::with_bus(1).expect("Couldn't start i2c. Is the interface enabled?");
    i2c.set_slave_address(ADDRESS_ADT7410).unwrap();

    loop {
       let temp = read_temperature(&i2c);
       println!("{}", temp);
       thread::sleep(Duration::from_millis(1000));
    }
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
