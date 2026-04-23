// Copyright 2026 harpertoken
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FirmwareError {
    #[error("Device not connected: {0}")]
    DeviceNotConnected(String),
    #[error("Communication error: {0}")]
    CommunicationError(String),
    #[error("Pin error: {0}")]
    PinError(String),
    #[error("Unsupported platform: {0}")]
    UnsupportedPlatform(String),
    #[error("IO error: {0}")]
    IoError(String),
}

pub type Result<T> = std::result::Result<T, FirmwareError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PinMode {
    Input,
    Output,
    InputPullUp,
    InputPullDown,
    Analog,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PinState {
    Low,
    High,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinConfig {
    pub pin_number: u32,
    pub mode: PinMode,
    pub initial_state: Option<PinState>,
}

#[async_trait]
pub trait GpioController: Send + Sync {
    async fn configure_pin(&self, config: PinConfig) -> Result<()>;
    async fn write_pin(&self, pin: u32, state: PinState) -> Result<()>;
    async fn read_pin(&self, pin: u32) -> Result<PinState>;
    async fn toggle_pin(&self, pin: u32) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct I2cConfig {
    pub address: u8,
    pub bus_speed: u32,
    pub sda_pin: Option<u32>,
    pub scl_pin: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct I2cMessage {
    pub address: u8,
    pub data: Vec<u8>,
}

#[async_trait]
pub trait I2cController: Send + Sync {
    async fn write(&self, message: I2cMessage) -> Result<()>;
    async fn read(&self, address: u8, length: usize) -> Result<Vec<u8>>;
    async fn write_read(&self, write_data: Vec<u8>, read_len: usize) -> Result<Vec<u8>>;
    async fn scan_devices(&self) -> Result<Vec<u8>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpiConfig {
    pub mode: SpiMode,
    pub frequency: u32,
    pub mosi_pin: Option<u32>,
    pub miso_pin: Option<u32>,
    pub clock_pin: Option<u32>,
    pub chip_select: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpiMode {
    Mode0,
    Mode1,
    Mode2,
    Mode3,
}

#[async_trait]
pub trait SpiController: Send + Sync {
    async fn transfer(&self, data: &[u8]) -> Result<Vec<u8>>;
    async fn write(&self, data: &[u8]) -> Result<()>;
    async fn read(&self, length: usize) -> Result<Vec<u8>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UartConfig {
    pub baud_rate: u32,
    pub data_bits: u8,
    pub stop_bits: u8,
    pub parity: Parity,
    pub rx_pin: Option<u32>,
    pub tx_pin: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Parity {
    None,
    Even,
    Odd,
}

#[async_trait]
pub trait UartController: Send + Sync {
    async fn write(&self, data: &[u8]) -> Result<usize>;
    async fn read(&self, length: usize) -> Result<Vec<u8>>;
    async fn write_string(&self, data: &str) -> Result<usize>;
    async fn flush(&self) -> Result<()>;
    async fn available(&self) -> Result<usize>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PwmConfig {
    pub pin: u32,
    pub frequency: u32,
    pub duty_cycle: f32,
}

#[async_trait]
pub trait PwmController: Send + Sync {
    async fn configure(&self, config: PwmConfig) -> Result<()>;
    async fn set_duty_cycle(&self, pin: u32, duty: f32) -> Result<()>;
    async fn set_frequency(&self, pin: u32, frequency: u32) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdcConfig {
    pub pin: u32,
    pub resolution: u8,
    pub vref: f32,
}

#[async_trait]
pub trait AdcController: Send + Sync {
    async fn read_voltage(&self, pin: u32) -> Result<f32>;
    async fn read_raw(&self, pin: u32) -> Result<u32>;
    async fn configure(&self, config: AdcConfig) -> Result<()>;
}

#[async_trait]
pub trait DelayController: Send + Sync {
    async fn delay_ms(&self, ms: u32);
    async fn delay_us(&self, us: u32);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub name: String,
    pub platform: Platform,
    pub firmware_version: Option<String>,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Platform {
    Esp32,
    Esp8266,
    Stm32,
    RaspberryPiPico,
    Arduino,
    Custom,
}

impl Platform {
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "esp32" => Platform::Esp32,
            "esp8266" => Platform::Esp8266,
            "stm32" => Platform::Stm32,
            "raspberrypipico" | "pico" => Platform::RaspberryPiPico,
            "arduino" => Platform::Arduino,
            _ => Platform::Custom,
        }
    }
}

#[async_trait]
pub trait FirmwareDevice: Send + Sync {
    fn device_info(&self) -> DeviceInfo;

    fn gpio(&self) -> Option<&dyn GpioController>;
    fn i2c(&self) -> Option<&dyn I2cController>;
    fn spi(&self) -> Option<&dyn SpiController>;
    fn uart(&self) -> Option<&dyn UartController>;
    fn pwm(&self) -> Option<&dyn PwmController>;
    fn adc(&self) -> Option<&dyn AdcController>;
    fn delay(&self) -> Option<&dyn DelayController>;

    async fn connect(&self) -> Result<()>;
    async fn disconnect(&self) -> Result<()>;
    async fn is_connected(&self) -> Result<bool>;
    async fn reset(&self) -> Result<()>;
}

pub struct FirmwareRegistry {
    devices: HashMap<String, Box<dyn FirmwareDevice>>,
}

impl FirmwareRegistry {
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: String, device: Box<dyn FirmwareDevice>) {
        self.devices.insert(name, device);
    }

    pub fn get(&self, name: &str) -> Option<&dyn FirmwareDevice> {
        self.devices
            .get(name)
            .map(|d| d.as_ref() as &dyn FirmwareDevice)
    }

    pub fn list_devices(&self) -> Vec<String> {
        self.devices.keys().cloned().collect()
    }
}

impl Default for FirmwareRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub mod esp32;
pub mod raspberry_pi;
pub mod stm32;

pub use esp32::Esp32Device;
pub use raspberry_pi::RaspberryPiDevice;
pub use stm32::Stm32Device;
