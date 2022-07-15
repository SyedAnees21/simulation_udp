use std::{net::UdpSocket,
    collections::VecDeque,
    time::{Duration,Instant},   
};
use serde::{Deserialize, Serialize};
use serde_json;
use std::str;

#[derive(Serialize,Debug)]
struct ClientResponse {
    packet_index: i16
}

#[derive(Clone,Serialize, Deserialize, Debug)]
struct Point {
    x: f64,
    y: f64,
    packet_index: i16
}


fn main() {
    
    let mut data_queue:VecDeque<Point> = VecDeque::new();
    let mut buf= [0; 40];

    let socket = UdpSocket::bind("127.0.0.1:8000").expect("Could not bind client socket");
    socket.connect("127.0.0.1:8888").expect("Could not connect to server");

     {
        let mut c_index:i16 = 0;

        for i in 0..400 {
            /*
            Starting reception from the Server
             */
            let len = socket.recv(&mut buf).expect("Could not get the datagram");
            let json_str = (str::from_utf8(&buf[..len]).expect("unable to parse")).to_string();
            let recieved_data:Point = serde_json::from_str(json_str.as_str()).unwrap();
            
            /*
            pushing the data queue 
            */
            data_queue.push_back(recieved_data);

            /*
            validation call to verify the packets in the list recieved
             */
            c_index = packet_validation( c_index, &socket , &mut data_queue , i);
        }

        println!("All Packets {:?}", data_queue.iter());
        println!("items {}", data_queue.len());
     }

    
}



fn packet_validation( mut index:i16, sock:&UdpSocket,  list:&mut VecDeque<Point>, i:usize) -> i16 { 
    match  list[i].packet_index - index {

        1 => {                                                           //if all OKAY!
            /*
            Acknowleding the server packets are intact!
            */ 
            index = list[i].packet_index;
                
            let response = ClientResponse{packet_index:0};
            let res_json = serde_json::to_string(&response).expect("Could not parse response");
            sock.send(res_json.as_bytes()).unwrap();
        },
        0 => {                                                           //if the same packet is recieved twice
            println!("Packet {}  recieved twice", list[i].packet_index);
        },
        _ => {                                                           //if the certain packet is missing
            /*
            Requesting the server for the missing packet
            */
            let mut buf= [0; 40];
    
            let response = ClientResponse{packet_index:list[i].packet_index-2};
            let res_json = serde_json::to_string(&response).expect("Could not parse response");
            sock.send(res_json.as_bytes()).unwrap();

            let len =sock.recv(&mut buf).expect("Could not get the datagram");
            let json_str = (str::from_utf8(&buf[..len]).expect("unable to parse")).to_string();

            let data: Point= serde_json::from_str(json_str.as_str()).unwrap();
            list.push_back(data);
        }
    }
    index
}

fn queue_management(start:Instant, dataqueue:&mut VecDeque<Point>) -> bool {
    if dataqueue.len() >= 500 {
        dataqueue.pop_front();
    }

    if start.elapsed() >= Duration::from_millis(500) {
        return true;
    }else {
        return false;
    }
}