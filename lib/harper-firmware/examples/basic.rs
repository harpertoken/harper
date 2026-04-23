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

use harper_firmware::{Esp32Device, FirmwareDevice, FirmwareRegistry};

fn main() {
    println!("Harper Firmware Crate Test\n");

    let esp = Esp32Device::new("/dev/ttyUSB0", "my-esp32");
    let info = esp.device_info();

    println!("Device: {}", info.name);
    println!("Platform: {:?}", info.platform);
    println!("Capabilities: {:?}", info.capabilities);

    let mut registry = FirmwareRegistry::new();
    registry.register("esp32-1".to_string(), Box::new(esp));

    println!("\nRegistered devices: {:?}", registry.list_devices());
    println!("\n✓ harper-firmware crate is working!");
}
