use serde_yaml;
use std::net::{TcpListener};
use std::io::{Read, Write};
use std::{str, thread, time::Duration, fs::File};
use crossbeam_channel::unbounded;
fn main() {
    let mut fault_al = Alarm {render_name: "Fault".to_string(), path: "/etc/EAS/faulted.yaml".to_string(), activators: vec!()};
    let mut general = Alarm {render_name: "General".to_string(), path: "/etc/EAS/gL.yaml".to_string(), activators: vec!()};
    let mut silent = Alarm {render_name: "Silent".to_string(), path: "/etc/EAS/sL.yaml".to_string(), activators: vec!()};
    let mut alarms = vec!(&mut fault_al, &mut general, &mut silent);
    let mut timeoutcount = 0;
    for i in &alarms{
        match std::fs::remove_file(&i.path){
            Ok(_) => {println!("removed old YAML file");},
            Err(_) => ()
        }
    }
    let (threadcom_s, threadcom_r) = unbounded();
    println!("starting");
    let listener = TcpListener::bind("0.0.0.0:5400").unwrap();
    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(mut streamm) => {
                    let mut data = [0 as u8; 50];
                    match streamm.read(&mut data){
                        Ok(size) => {
                        match str::from_utf8(&data[0..size]){
                            Ok(string_out) => {
                                let s: String = (&string_out).to_string();
                                let msg: Vec<u8>;
                                println!("Got data: {}", s);
                                match threadcom_s.send(s){Ok(_)=> msg = b"ok".to_vec(), Err(e)=>{println!("{}",e); msg=b"fault".to_vec();}}
                                match streamm.write(&msg.as_slice()) {
                                Ok(_) => {println!("Write success")},
                                Err(e) => {println!("Write Error: {}", e)}
                                }}
                            Err(_) => {println!("fault in listening"); break;}
                        }}
                        Err(_) => {println!("Fault when reading data!"); break;}
                    }}
                Err(e) => {println!("Connection failed with code {}", e);thread::sleep(Duration::from_secs(1));}
            }}
    });
    loop{
        println!("main loop run");
        thread::sleep(Duration::from_millis(500));
        match threadcom_r.try_recv(){
            Ok(out) => {
                timeoutcount = 0;
                let mut e = out.split(' ');
                match e.nth(0) {Some(alm)=>{
                for i in &mut alarms{
                    if alm.eq(&i.render_name){
                        match e.next(){Some(activator)=>{
                        if activator.eq("clear"){i.clear();}
                        else{i.add(activator.to_string());}
                        } None=>()}
                    }}}None=>{println!("bad alarm data");}
                }}
            Err(_) => {thread::sleep(Duration::from_secs(2));
            println!("no new data, sleeping");
            timeoutcount += 1;
            if timeoutcount > 20{
                for i in &mut alarms{i.clear()}
            }} //usually will return an error as no data has been sent
        }
        for i in &mut alarms{
            match i.update(){
                Ok(_) => (),
                Err(e) => {println!("Can't write to yaml, {}", e)}
            }}
    }
}
struct Alarm{
    render_name: String,
    path: String,
    activators: Vec<String>
}
impl Alarm{
    fn update(&mut self) -> std::io::Result<()>{
        let mut file = File::create(&self.path)?;
        let s = serde_yaml::to_string(&self.activators).unwrap();
        println!("{}",s);
        file.write_all(s.as_bytes())?;
        Ok(())
    }
    fn clear(&mut self){self.activators.clear();}
    fn add(&mut self, act: String){
        if !self.activators.contains(&act) 
        {self.activators.push(act);}
    }
}
