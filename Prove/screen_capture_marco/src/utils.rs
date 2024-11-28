use ipnetwork::IpNetwork;
use pnet::datalink;
use std::io::BufRead;

pub const STREAMERS_LIST_PATH : &str = "../config/streamers_list.txt";

pub fn is_ip_in_lan(ip_to_check: &str) -> bool {
    // Converti l'indirizzo target in un oggetto IPv4
    let target_ip: std::net::Ipv4Addr = ip_to_check
        .parse()
        .expect("Indirizzo IP non valido");

    // Ottieni le interfacce di rete
    let interfaces = datalink::interfaces();

    for interface in interfaces {
        for ip in interface.ips {
            // Controlla solo gli indirizzi IPv4
            if let IpNetwork::V4(network) = ip {
                if network.contains(target_ip) {
                    return true; // L'indirizzo appartiene alla subnet
                }
            }
        }
    }

    false // Nessuna corrispondenza trovata
}

pub fn get_streamers_map() -> std::collections::HashMap<String, String> {
    let mut streamers_map = std::collections::HashMap::new();
    let file = std::fs::File::open(STREAMERS_LIST_PATH).expect("File non trovato");
    let mut reader = std::io::BufReader::new(file);
    let mut line = String::new();

    while let Ok(_) = reader.read_line(&mut line) {
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if parts.len() != 2 {
            break;
        }
        println!("{:?}", parts);
        let key = parts[0].to_string();
        let value = parts[1].to_string();
        streamers_map.insert(key, value);
        line.clear();
    }

    println!("{:?}", streamers_map);
    streamers_map

}