// use embedded_hal::i2c::{I2c};
use esp_hal::{i2c::master::*, Blocking};
const XPOWERS_AXP2101_PMU_STATUS1: u8 = 0x00;
const XPOWERS_AXP2101_PMU_STATUS2: u8 = 0x01;
// Seems that PMU Led control is inverted
const PMU_LED_ON: u8 = 0x35;
const XPOWERS_AXP2101_CHGLED_SET_CTRL: u8 = 0x69;
const XPOWERS_AXP2101_DC_VOL3_CTRL: u8 = 0x85;

const ON_LED_COMMAND: [u8; 2] = [XPOWERS_AXP2101_CHGLED_SET_CTRL, PMU_LED_ON];
const XPOWERS_AXP2101_DCDC4_VOL1_MIN: u16 = 500;
const XPOWERS_AXP2101_DCDC4_VOL_STEPS1: u8 = 10;

pub struct AldoBank<const ALDO_NUMBER: usize> {
    // max_voltage: [u16; ALDO_NUMBER],
    min_voltage: [u16; ALDO_NUMBER],
    steps_mv: [u16; ALDO_NUMBER],
    voltage_address_control: [u8; ALDO_NUMBER],
    enable_address: u8,
}

impl<const T: usize> AldoBank<T> {
    //     pub fn check_input_voltage(
    //         &self,
    //         aldo_number: usize,
    //         voltage_in_mv: u16,
    //     )
    //     -> Result<(), u8>
    //     {
    //         if voltage_in_mv > self.max_voltage[aldo_number] {
    //             return Err(format!(
    //                 "Input max voltage is {}",
    //                 self.max_voltage[aldo_number]
    //             ));
    //         }
    //         if voltage_in_mv < self.min_voltage[aldo_number] {
    //             return Err(format!(
    //                 "Input min voltage  is {}",
    //                 self.min_voltage[aldo_number]
    //             ));
    //         }
    //         if voltage_in_mv % self.steps_mv[aldo_number] != 0 {
    //             return Err(format!(
    //                 "Input voltage should be a multiple of {}",
    //                 self.steps_mv[aldo_number]
    //             ));
    //         }
    //         Ok(())
    //     }

    pub fn compute_voltage_command(&self, aldo_number: usize, voltage_in_mv: u16) -> u8 {
        let scale_from_zero = voltage_in_mv - self.min_voltage[aldo_number];
        (scale_from_zero / self.steps_mv[aldo_number]) as u8
    }

    pub fn check_aldo_number(&self, aldo_number: usize) -> Result<(), u8> {
        if aldo_number > T {
            return Err(
                // format!("Aldo max number is {}", T)
                0,
            );
        }
        Ok(())
    }
}

const AXP210_ALDO_NUMBER: usize = 4;
const AXP210_ALDOS: AldoBank<AXP210_ALDO_NUMBER> = AldoBank {
    // max_voltage: [3500; AXP210_ALDO_NUMBER],
    min_voltage: [500; AXP210_ALDO_NUMBER],
    steps_mv: [100; AXP210_ALDO_NUMBER],
    voltage_address_control: {
        const LDO_VOL_CONTROL_BASE_REGISTER_ADDR: u8 = 0x92;
        let mut values: [u8; AXP210_ALDO_NUMBER] =
            [LDO_VOL_CONTROL_BASE_REGISTER_ADDR; AXP210_ALDO_NUMBER];
        let mut i: usize = 0;
        while i < AXP210_ALDO_NUMBER {
            values[i] += i as u8;
            i += 1;
        }
        values
    },
    enable_address: 0x90,
};

pub struct Pmu<'a> {
    pub address: u8,
    pub i2c: I2c<'a, Blocking>,
}

impl<'a> Pmu<'a> {
    pub fn new(i2c: I2c<'a, Blocking>, address: u8) -> Self {
        Self { address, i2c }
    }

    fn write(&mut self, bytes: &[u8]) -> Result<(), u8> {
        match self.i2c.write(self.address, bytes) {
            Ok(()) => Ok(()),
            Err(_e) => Err(1),
        }
    }

    fn set_aldo_voltage(&mut self, aldo_number: usize, voltage_in_mv: u16) -> Result<(), u8> {
        // match AXP210_ALDOS.check_aldo_number(aldo_number) {
        //     Ok(_v) => {}
        //     Err(_e) => return Err(_e),
        // }

        // match AXP210_ALDOS.check_input_voltage(aldo_number, voltage_in_mv) {
        //     Ok(_v) => {}
        //     Err(_e) => return Err(_e),
        // }

        let temp: [u8; 2] = [
            AXP210_ALDOS.voltage_address_control[aldo_number],
            AXP210_ALDOS.compute_voltage_command(aldo_number, voltage_in_mv),
        ];

        match self.write(&temp) {
            Ok(_v) => Ok(()),
            Err(_e) => Err(1), // Err(_e) => Err(format!(
                               //     "Could not write properly at register {:#04x} the value {:?}",
                               //     AXP210_ALDOS.voltage_address_control[aldo_number], temp[1]
                               // )),
        }
    }

    fn enable_aldo(&mut self, aldo_number: usize) -> Result<(), u8> {
        match AXP210_ALDOS.check_aldo_number(aldo_number) {
            Ok(_v) => {}
            Err(_e) => return Err(_e),
        }
        let temp: [u8; 2] = [AXP210_ALDOS.enable_address, 1 << aldo_number];
        match self.write(&temp) {
            Ok(_v) => Ok(()),
            Err(_e) => Err(1),
        }
    }

    fn set_dc4_voltage(&mut self) -> Result<bool, ()> {
        /* To have 3.3 for main voltage
         */

        /* 1.2 V, step is 10 mV */
        let value_to_write: u16 =
            (1200 - XPOWERS_AXP2101_DCDC4_VOL1_MIN) / XPOWERS_AXP2101_DCDC4_VOL_STEPS1 as u16;

        let temp: [u8; 2] = [XPOWERS_AXP2101_DC_VOL3_CTRL, value_to_write as u8];

        match self.write(&temp) {
            Ok(_v) => Ok(true),
            Err(_e) => {
                panic!(
                    "Problem Setting DC VOL 3  voltage with {:?}, error : {:?}",
                    temp[0], _e
                )
            }
        }
    }

    pub fn start_pmu(&mut self) -> Result<bool, ()> {
        let mut temp: [u8; 1] = [0];
        match self
            .i2c
            .write_read(self.address, &[XPOWERS_AXP2101_PMU_STATUS1], &mut temp)
        {
            Ok(_v) => {
                if temp[0] != 32 {
                    panic!("VBUS should be good")
                }
            }
            Err(_e) => {}
        }
        match self
            .i2c
            .write_read(self.address, &[XPOWERS_AXP2101_PMU_STATUS2], &mut temp)
        {
            Ok(_v) => {
                if temp[0] & 16 == 0 {
                    panic!("System should be powered on")
                }
            }
            Err(_e) => {}
        }

        match self.write(&ON_LED_COMMAND) {
            Ok(_v) => {}
            Err(_e) => {}
        }
        match self
            .i2c
            .write_read(self.address, &[XPOWERS_AXP2101_CHGLED_SET_CTRL], &mut temp)
        {
            Ok(_v) => {
                if temp[0] != 0x35 {
                    panic!("workwith {:?}", temp)
                }
            }
            Err(_e) => {}
        }

        match self.set_dc4_voltage() {
            Ok(_v) => {}
            Err(_e) => {
                panic!("Problem Setting DC4 Voltage")
            }
        }
        match self.set_aldo_voltage(0, 3300) {
            Ok(_v) => {}
            Err(_e) => {
                panic!("Problem Setting Aldo 1 voltage")
            }
        }
        match self.set_aldo_voltage(1, 3300) {
            Ok(_v) => {}
            Err(_e) => {
                panic!("Problem Setting Aldo 2 voltage")
            }
        }
        match self.set_aldo_voltage(2, 2500) {
            Ok(_v) => {}
            Err(_e) => {
                panic!("Problem Setting Aldo 3 voltage")
            }
        }
        match self.set_aldo_voltage(3, 1800) {
            Ok(_v) => {}
            Err(_e) => {
                panic!("Problem Setting Aldo 4 voltage")
            }
        }

        match self.enable_aldo(0) {
            Ok(_v) => {}
            Err(_e) => {
                panic!("Problem enabling Aldo 0")
            }
        }
        match self.enable_aldo(1) {
            Ok(_v) => {}
            Err(_e) => {
                panic!("Problem enabling Aldo 1")
            }
        }
        match self.enable_aldo(2) {
            Ok(_v) => {}
            Err(_e) => {
                panic!("Problem enabling Aldo 2")
            }
        }
        match self.enable_aldo(3) {
            Ok(_v) => {}
            Err(_e) => {
                panic!("Problem enabling Aldo 3")
            }
        }
        Ok(true)
    }
}
