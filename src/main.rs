use mac_address::mac_address_by_name;  
use std::result::Result;  
use simple_redis;
use std::process::Command;   
use clap::{App, Arg}; 

fn main() -> Result<(), Box<dyn std::error::Error>> { 
    let matches = App::new("nicgetmac")  
        .version("1.0.0")  
        .author("h13317136163@163.com")  
        .about("MAC地址采集程序-网卡版")  
        .arg(  
            Arg::with_name("ip")  
                .short("i")  
                .long("ip")  
                .value_name("IP_ADDRESS")  
                .help("Redis数据库地址 例: redis://127.0.0.1:6379/0")  
                .takes_value(true)  
                .required(true),  
        )  
        .arg(  
            Arg::with_name("network_name")  
                .short("n")  
                .long("network")  
                .value_name("name")  
                .help("Network card name")  
                .takes_value(true)  
                .multiple(true)
                .required(true)
                .required(true),  
        ) 
        .get_matches();  
    let ip_address = matches.value_of("ip").unwrap(); 
    let network_names: Vec<&str> = matches.values_of("network_name").unwrap().collect();
    let serial_number = get_bios_serial_number()?;  
    let mut mac_found = false;
    println!("Redis address: {}", ip_address);  
    println!("SN: {}", serial_number); 
    let mut mac_addresses = Vec::new(); 
    for iface in network_names {
        match mac_address_by_name(iface) {
            Ok(Some(mac)) => {
                mac_addresses.push(mac.to_string());
                mac_found = true;
            }
            Ok(None) => {
                eprintln!("Interface \"{}\" not found", iface);
            }
            Err(e) => {
                eprintln!("Error fetching MAC address for \"{}\": {}", iface, e);
            }
        }
    }
    if mac_found { // 只有当找到MAC地址时才执行下面的打印和Redis操作  
        println!("MAC addresses: {}", mac_addresses.join(" "));  
        let macs_joined2: String = mac_addresses.join(" ");  
        let native_options = NativeOptions::default();  
        let _ = run_native("My egui App", native_options, Box::new(move |cc| {  
        Box::new(MyEguiApp::new(cc, &macs_joined2))  
        }));  
        // Redis 操作代码块开始  
        match simple_redis::create(ip_address) {  
            Ok(mut client) => {  
                match client.set(&*serial_number, &*macs_joined2) {  
                    Ok(_) => println!("MAC addresses set in Redis."),  
                    Err(error) => println!("Unable to set value in Redis: {}", error),  
                }  
                match client.quit() {  
                    Ok(_) => println!("退出数据库."),  
                    Err(error) => println!("Error: {}", error),  
                }  
            }, // Redis 客户端操作结束  
            Err(error) => {  
                println!("Unable to create Redis client: {}", error);  
            }  
        } 
    } 
    Ok(())  
}  

//获取序列号
fn get_bios_serial_number() -> Result<String, Box<dyn std::error::Error>> {  
    let output = Command::new("wmic")  
        .arg("bios")  
        .arg("get")  
        .arg("serialnumber")  
        .output()?;   
    let stdout = String::from_utf8_lossy(&output.stdout);  
    let lines: Vec<&str> = stdout.lines().collect();  
    let serial_line = lines.get(1); 
    if let Some(serial_line) = serial_line {  
        let serial_number_part = serial_line.split_whitespace().last();
        if let Some(serial_number) = serial_number_part {  
            return Ok(serial_number.to_string());  
        }  
    }  
    Err(format!("Failed to find BIOS serial number in WMIC output: {}", stdout).into())  
}  
