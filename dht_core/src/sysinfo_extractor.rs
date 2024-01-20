use quick_xml::writer::Writer;
use std::io::Cursor;
use sysinfo::{CpuRefreshKind, Disks, MemoryRefreshKind, Networks, RefreshKind, System};

pub(crate) fn get_record() -> Vec<u8> {
    match serialize_xml() {
        Ok(res) => res, 
        Err(e) => panic!("Error while writing sysinfo in XML {e}"),
    }
}

fn write_cpus_info(writer: &mut Writer<Cursor<Vec<u8>>>) -> Result<(), quick_xml::Error> {
    let mut sys =
        System::new_with_specifics(RefreshKind::new().with_cpu(CpuRefreshKind::everything()));

    // Wait a bit because CPU usage is based on diff.
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    // Refresh CPUs again.
    sys.refresh_cpu();

    writer
        .create_element("cpus")
        .write_inner_content::<_, quick_xml::Error>(|writer| {
            for cpu in sys.cpus() {
                writer
                    .create_element("cpu")
                    .with_attribute(("name", cpu.name()))
                    .with_attribute(("usage", cpu.cpu_usage().to_string().as_str()))
                    .with_attribute(("frequency", cpu.frequency().to_string().as_str()))
                    .write_empty()?;
            }
            Ok(())
        })?;

    Ok(())
}

fn write_ram_info(writer: &mut Writer<Cursor<Vec<u8>>>) -> Result<(), quick_xml::Error> {
    let mut sys =
        System::new_with_specifics(RefreshKind::new().with_memory(MemoryRefreshKind::everything()));

    sys.refresh_memory();

    writer
        .create_element("ram")
        .with_attribute(("total", sys.total_memory().to_string().as_str()))
        .with_attribute(("used", sys.used_memory().to_string().as_str()))
        .with_attribute(("total_swap", sys.total_swap().to_string().as_str()))
        .with_attribute(("used_swap", sys.used_swap().to_string().as_str()))
        .write_empty()?;

    Ok(())
}

fn write_disks_info(writer: &mut Writer<Cursor<Vec<u8>>>) -> Result<(), quick_xml::Error> {
    let disks = Disks::new_with_refreshed_list();

    writer
        .create_element("disks")
        .write_inner_content::<_, quick_xml::Error>(|writer| {
            for disk in disks.list() {
                writer
                    .create_element("disk")
                    .with_attribute((
                        "name",
                        match disk.name().to_str() {
                            Some(s) => s,
                            None => "",
                        },
                    ))
                    .with_attribute(("kind", disk.kind().to_string().as_str()))
                    .with_attribute((
                        "file_system",
                        match disk.file_system().to_str() {
                            Some(fs) => fs,
                            None => "",
                        },
                    ))
                    .with_attribute(("total_space", disk.total_space().to_string().as_str()))
                    .with_attribute((
                        "available_space",
                        disk.available_space().to_string().as_str(),
                    ))
                    .write_empty()?;
            }
            Ok(())
        })?;

    Ok(())
}

fn write_networks_info(writer: &mut Writer<Cursor<Vec<u8>>>) -> Result<(), quick_xml::Error> {
    let mut networks = Networks::new_with_refreshed_list();

    std::thread::sleep(std::time::Duration::from_secs(1));
    networks.refresh_list();

    let mut res = vec![];

    for (interface_name, network) in &networks {
        res.push((
            interface_name.to_owned(),
            network.received(),
            network.total_received(),
            network.transmitted(),
            network.total_transmitted(),
        ));
    }

    writer
        .create_element("networks")
        .write_inner_content::<_, quick_xml::Error>(|writer| {
            for (interface_name, network) in &networks {
                writer
                    .create_element("network")
                    .with_attribute(("name", interface_name.as_str()))
                    .with_attribute(("received", network.received().to_string().as_str()))
                    .with_attribute((
                        "total_received",
                        network.total_received().to_string().as_str(),
                    ))
                    .with_attribute(("transmitted", network.transmitted().to_string().as_str()))
                    .with_attribute((
                        "total_transmitted",
                        network.total_transmitted().to_string().as_str(),
                    ))
                    .write_empty()?;
            }
            Ok(())
        })?;

    Ok(())
}

fn serialize_xml() -> Result<Vec<u8>, quick_xml::Error> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    // Root node
    writer
        .create_element("record")
        .write_inner_content::<_, quick_xml::Error>(|writer| {
            write_cpus_info(writer)?;
            write_ram_info(writer)?;
            write_disks_info(writer)?;
            write_networks_info(writer)?;
            Ok(())
        })?;

    let result = writer.into_inner().into_inner();

    Ok(result)
}
