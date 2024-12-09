Note, pete used an active buzzer - buzzer has inbuilt oscillator so produces a pure tone as soon as 5V signal provided

# SPI bus 3
- SCK - PB3
- MISO - PB4
- MOSI - PB5

## Memory (W25Q128JVSIQ)
- CS - PC13

## SD Card
stm32f302cb does not have a dedicated sd card host controller => limited to one bit mode (SPI mode)
- CS: PC14

# SPI Bus 2
- SCK: 13
- MISO: 14
- MOSI: 15

## Accelerometer 2 (ADXL375)
- CS: PB2
- INT_1: 
- INT_2: PA10