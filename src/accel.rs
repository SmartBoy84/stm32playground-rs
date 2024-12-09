fn accel_test() {
    {
        info!("resetting and testing adxl");

        // ADXL test
        let spi_2 = spi_2.get_mut();

        // ADXL "driver" from datasheet + here: https://github.com/adafruit/Adafruit_ADXL375/blob/master/Adafruit_ADXL375.cpp
        accel2_cs.set_low(); // enable forever

        // reset configs as per register map

        let write_buf = [(0x31u8 & 0x7Fu8), 0];

        spi_2.blocking_write(&write_buf).unwrap();

        for register in 0x1du8..=0x2a {
            spi_2.blocking_write(&[register, 0u8]).unwrap();
        }

        let write_buf = [(0x2Cu8 & 0x7Fu8), 0x0a];
        spi_2.blocking_write(&write_buf).unwrap();

        for register in 0x2du8..=0x2f {
            spi_2.blocking_write(&[register, 0u8]).unwrap();
        }

        let write_buf = [(0x38u8 & 0x7Fu8), 0];
        spi_2.blocking_write(&write_buf).unwrap();

        // get device id
        let mut read_buf = [0u8, 0u8];
        let write_buf = [(0x0 & 0x7Fu8 | 0x80u8), 0u8];
        spi_2.blocking_transfer(&mut read_buf, &write_buf).unwrap();
        assert_eq!(read_buf[0], 0b11100101);

        info!("Successfully connected to accelerometer!");
    }
}