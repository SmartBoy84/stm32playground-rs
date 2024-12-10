and ill have to give you a system architecture but basically neptunium has two spi busses, one has the lsm6dsox, adxl375 and lps22hb sensors (all with interrupts connected to stm32). and one spi bus with sx1262, a nor flash chip and sd card. then there's a uart connected to a gps receiver.

Note, pete used an active buzzer - buzzer has inbuilt oscillator so produces a pure tone as soon as 5V signal provided

# SPI bus 3
- SCK - PB3
- MISO - PB4
- MOSI - PB5

## Memory (W25Q128JVSIQ)
- CS - PC13

## SD Card
stm32f302cb does not have a dedicated sd card host controller => limited to one bit mode (SPI mode)
God damn! This is quite large - I needed to switch to `--release` flag to get it to fit  
- CS: PC14

## LoRa module
E22-900MM22S => SX1262  
- CS: PB6

# SPI Bus 2
- SCK: 13
- MISO: 14
- MOSI: 15

## Accelerometer 2 (ADXL375)
- CS: PB2
- INT_1: 
- INT_2: PA10