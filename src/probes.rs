use sys_info::*;

pub enum Probe {
    CPUNumber(u32),
    LoadAvg(f64, f64, f64),
    MemInfo(u64, u64, u64, u64, u64),
    DiskInfo(u64, u64),
}

pub fn get_probes() -> Vec<Probe> {
    let mut vec = Vec::with_capacity(1);
    match cpu_num() {
        Ok(stats) => {
            vec.push(Probe::CPUNumber(stats));
        }
        Err(x) => println!("\nNumber CPUs statistics error: {}", x.to_string()),
    }
    match loadavg() {
        Ok(load) => {
            vec.push(Probe::LoadAvg(load.one, load.five, load.fifteen));
        }
        Err(x) => println!("LoadAvg statistics error: {}", x.to_string()),
    }
    match mem_info() {
        Ok(mem) => {
            vec.push(Probe::MemInfo(mem.total, mem.free, mem.avail, mem.buffers, mem.cached));
        }
        Err(x) => println!("LoadAvg statistics error: {}", x.to_string()),
    }
    match disk_info() {
        Ok(disk) => {
            vec.push(Probe::DiskInfo(disk.total, disk.free));
        }
        Err(x) => println!("LoadAvg statistics error: {}", x.to_string()),
    }
    vec
}
