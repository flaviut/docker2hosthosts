extern crate shiplift;
extern crate clap;

use shiplift::{Docker, builder, Container, rep};
use clap::{Arg, App};
use std::{thread, time, fs};
use std::io::{Write, Read};

fn get_container_ips<'a>(container: &'a rep::ContainerDetails) -> impl Iterator<Item = &'a String> {
    return container.NetworkSettings.Networks.values()
        .map(|network| &network.IPAddress);
}

fn container_to_exposed_hostname(container: &rep::ContainerDetails) -> String {
    if container.Config.Domainname != "" {
        // we have a domain
        return format!("{}.{}", container.Config.Hostname,
                       container.Config.Domainname);
    } else {
        return format!("{}.docker.local", container.Config.Hostname);
    }
}

fn container_to_hosts_entries(container: &rep::ContainerDetails) -> String {
    let domain = container_to_exposed_hostname(container);
    return get_container_ips(container)
        .map(|ip| format!("{}\t{}", ip, domain))
        .collect::<Vec<_>>()
        .join("\n");
                             
}

fn generate_hosts(docker: &Docker) -> Result<String, shiplift::Error> {
    return Ok(docker.containers()
              .list(&builder::ContainerListOptions::default())?
              .iter()
              .map(|container_rep| Container::new(&docker, container_rep.Id.clone()).inspect())
              .collect::<Result<Vec<_>, _>>()?
              .iter()
              .map(|container| container_to_hosts_entries(&container))
              .collect::<Vec<_>>()
              .join("\n") + "\n");
}

const HOSTS_HEADER: &str= "# ===DOCKER HOSTS===\n";

fn update_hosts(docker: &Docker, output_file: &String) -> Result<(), shiplift::Error> {
    let mut existing_contents = String::new();

    {
        match fs::File::open(&output_file)
            .map(|mut file| file.read_to_string(&mut existing_contents)) {
             _ => (),
        };
    }

    let mut terminator_slices = existing_contents.split(&HOSTS_HEADER);
    let generated_hosts = generate_hosts(&docker)?.into_bytes();
    let mut file = fs::File::create(&output_file)?;
    file.write_all(terminator_slices.next().unwrap_or(&String::default()).as_bytes())?;
    file.write_all(HOSTS_HEADER.as_bytes())?;
    file.write_all(&generated_hosts)?;
    terminator_slices.next();  // discard the old contents
    file.write_all(HOSTS_HEADER.as_bytes())?;
    file.write_all(terminator_slices.next().unwrap_or(&String::default()).as_bytes())?;
    return Ok(());
}

fn wait_for_containers(docker: &Docker, output_file: String) -> Result<(), shiplift::Error> {
    let upcoming = docker.events(&builder::EventsOptions::default())?
                         .filter(|event| match event.status {
                             Some(ref s) if s == "start" || s == "stop" => true,
                             _ => false
                         });

    update_hosts(&docker, &output_file)?;
    println!("Completed initial hosts update. Waiting for events...");
    // TODO Fix this when events() returns Results instead of panicing
    for event in upcoming {
        update_hosts(&docker, &output_file)?;
        println!("Updated hosts ({} {})", event.status.unwrap(),
                 event.from.unwrap_or("UKNOWN".to_string()));
    }

    unreachable!();
}

fn main() {
    let args = App::new("docker2hosthosts")
        .author("Flaviu Tamas <me@flaviutamas.com>")
        .about("Generates a hosts file pointing to started docker containers.
               Can be used with either dnsmasq for global DNS resolution, or \
               just pointed at /etc/hosts for local use. Containers that have \
               a domain will be given the name <domain>.<hostname>, otherwise \
               containers are accessible from <hostname>.docker.local.")
        .arg(Arg::with_name("output")
             .short("o").long("output")
             .value_name("HOSTS_FILE")
             .help("Location to render the hosts file. Existing entries will \
                   not be overwriten")
             .default_value("/etc/hosts")
             .takes_value(true))
        .get_matches();

    let docker = Docker::new();

    loop {
        match wait_for_containers(&docker,
                            args.value_of("output").unwrap().to_string()) {
            Err(error) => println!("Something ({:?}) failed; retrying in 1 second", error),
            Ok(()) => return,
        }
        thread::sleep(time::Duration::from_secs(1));
    }
}
