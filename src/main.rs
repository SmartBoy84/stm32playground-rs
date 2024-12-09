#![no_std]
#![no_main]

use core::cell::RefCell;

use defmt::info;
use embassy_executor::{main, Spawner};
// embedded_sdmmc is blocking
use embassy_stm32::{
    dma::NoDma,
    gpio::{Input, Level, Output, Pull, Speed},
    spi::{self},
};
use embassy_sync::blocking_mutex::{raw::NoopRawMutex, Mutex};
use embedded_sdmmc::{BlockDevice, TimeSource, Timestamp, VolumeIdx, VolumeManager};
// TODO; look at crates.md for warning
use {defmt_rtt as _, panic_probe as _};

// Entry point
#[main]
async fn main(_spawner: Spawner) {
    let dp = embassy_stm32::init(embassy_stm32::Config::default()); // to get device peripherals -  same as .take() for embedded_hal => cannot call more than once!

    // ask pete about gpio::Speed - https://medium.com/@aliaksandr.kavalchuk/the-gpio-output-speed-whats-behind-this-parameter-dcceaa351875
    // is there no standard active state (high/low) for spi devices?
    info!("Initialising cs psins");
    // chip selects - set all to inactive state
    let _accel1_cs = Output::new(dp.PC15, Level::High, Speed::High);
    let mut accel2_cs = Output::new(dp.PB2, Level::High, Speed::High);
    let mut sd_cs = Output::new(dp.PC14, Level::High, Speed::High);
    let _radio_cs = Output::new(dp.PB6, Level::High, Speed::High);
    let _mem_cs = Output::new(dp.PC13, Level::High, Speed::High);
    let _baro_cs = Output::new(dp.PA9, Level::High, Speed::High);

    let _accel2_int2 = Input::new(dp.PA10, Pull::Down); // active high

    let mut spi_2_config = spi::Config::default(); // todo; look at config options - 1MHz is default => what's max? also lsb/msb first (default matches adafruit impl)
    spi_2_config.mode = spi::MODE_3; // https://www.analog.com/en/resources/analog-dialogue/articles/introduction-to-spi-interface.html

    // IMPORTANT - look at [crates.md] - is NoopRawMutex logical here? Also blocking_mutex (not sync) for embedded_hal interop - what's the diff
    let mut spi_2 = Mutex::<NoopRawMutex, _>::new(spi::Spi::new(
        dp.SPI2, // 3,4,5 connected to SPI bus 3
        dp.PB13,
        dp.PB15,
        dp.PB14,
        NoDma, // uhh, does it matter what channel I use?
        NoDma,
        spi_2_config,
    ));

    let spi_3 = Mutex::<NoopRawMutex, _>::new(RefCell::new(spi::Spi::new(
        dp.SPI3,
        dp.PB3,
        dp.PB5,
        dp.PB4,
        NoDma,
        NoDma,
        spi::Config::default(),
    )));

    // 3.11: Direct memory access (DMA) channels allow data buffering with no cpu overhead(?), necessary for async => todo; read more
    // mean that MCU itself doesn't have to waste time reading - interrupt raised once DMA buffer filled

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

    // stm32f302cb does not have a dedicated sd card host controller - limited to one bit mode (SPI mode)
    sd_cs.set_high();
    let sd_spi = embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice::new(&spi_3, sd_cs);
    // let sd_spi = embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice::new(&spi_3, sd_cs);

    // ez pz - this struct manages the enable pin (configurable), and bus mutex for us!
    // use SpiDeviceWithConfig if Cs active state (or write freq) varies for different devices

    let sd_card = embedded_sdmmc::SdCard::new(sd_spi, embassy_time::Delay);
    // embassy_time::Delay is just a bridge between embassy_time and embedded_hal traits

    // holy fragole it compiled ma!
    info!(
        "Connected to {:?}; {:?} bytes; {:?} blocks",
        sd_card
            .get_card_type()
            .expect("Failed to connect to sd card"),
        sd_card.num_blocks().unwrap(),
        sd_card.num_bytes().unwrap()
    );
    // sd card benchmark - https://www.jblopen.com/sd-card-benchmarks/

    // let rtc = rtc::Rtc::new(dp.RTC, RtcConfig::default()); // <-- this is not accurate at all so for now...
    // let vol_mgr = VolumeManager::new(block_device, time_source);
    struct DummyTimeSource;
    impl TimeSource for DummyTimeSource {
        fn get_timestamp(&self) -> embedded_sdmmc::Timestamp {
            Timestamp {
                year_since_1970: 0,
                zero_indexed_month: 0,
                zero_indexed_day: 0,
                hours: 0,
                minutes: 0,
                seconds: 0,
            }
        }
    }
    // VolumeManager needs this to write correct timestamps on files - pshaw! who needs timestamps
    let mut vol_mgr = VolumeManager::new(sd_card, DummyTimeSource);
    let mut vol0 = vol_mgr
        .open_volume(VolumeIdx(0))
        .expect("No volumes on sd card");

    let mut root_dir = vol0.open_root_dir().unwrap();
    let mut test_file = root_dir
        .open_file_in_dir("test.txt", embedded_sdmmc::Mode::ReadOnly)
        .unwrap();

    // alas, no allocator - bit by bit reading
    // let mut s = String::<15>::new(); // Hello world!
    // unsafe { test_file.read(&mut s.as_bytes_mut()).unwrap() };
    // info!("Read: {}", s.as_str());

    while !test_file.is_eof() {
        let mut buffer = [0u8; 32];
        let num_read = test_file.read(&mut buffer).unwrap();
        for b in &buffer[0..num_read] {
            info!("{}", *b as char);
        }
    }

    // loop {
    //     info!("{:?}", rtc.now().unwrap().second());
    //     Timer::after_secs(1).await;
    // }
    // let vol_mgr = VolumeManager::new(sd_card, TimeSource);
}
