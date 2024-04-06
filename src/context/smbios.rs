use super::{SmBiosMap, SmBiosMapExt, SmBiosType};
use smbioslib::*;
use std::any::type_name;

/// Fills in SMBIOS fields from real hardware, falling back to defaults when unavailable.
/// Default values taken from https://docs.vrchat.com/docs/using-vrchat-in-a-virtual-machine#qemu
pub fn populate_auto(map: &mut SmBiosMap) {
	let Ok(dmi) = table_load_from_device() else {
		panic!("unable to load dmi tables, cannot retrieve smbios parameters");
	};

	populate_bios_information(map, get_table::<SMBiosInformation>(&dmi));
	populate_system_information(map, get_table::<SMBiosSystemInformation>(&dmi));
	populate_baseboard_information(map, get_table::<SMBiosBaseboardInformation>(&dmi));
	populate_enclosure_information(map);
	populate_processor_information(map, get_table::<SMBiosProcessorInformation>(&dmi));
	populate_oem_strings(map);
	populate_memory_device(map, get_table::<SMBiosMemoryDevice>(&dmi));
}

fn populate_bios_information(map: &mut SmBiosMap, table: Option<SMBiosInformation>) {
	let version = get_string_field(&table, |t| t.version(), "F31o");
	let vendor = get_string_field(&table, |t| t.vendor(), "American Megatrends International, LLC.");
	let release_major = get_field(&table, |t| t.system_bios_major_release(), 5);
	let release_minor = get_field(&table, |t| t.system_bios_minor_release(), 17);
	let date = get_string_field(&table, |t| t.release_date(), "12/03/2020");

	map.add_fields(
		SmBiosType::BiosInformation,
		vec![
			("version", version),
			("vendor", vendor),
			("uefi", String::from("on")),
			("release", format!("{release_major}.{release_minor}")),
			("date", date),
		],
	);
}

fn populate_system_information(map: &mut SmBiosMap, table: Option<SMBiosSystemInformation>) {
	let version = get_string_field(&table, |t| t.version(), "-CF");
	let sku = get_string_field(&table, |t| t.sku_number(), "Default string");
	let product = get_string_field(&table, |t| t.product_name(), "X570 AORUS ULTRA");
	let manufacturer = get_string_field(&table, |t| t.manufacturer(), "Gigabyte Technology Co., Ltd.");
	let uuid = get_uuid_field(&table, |t| t.uuid(), "3137f3a5-8fa3-41a4-87f5-aadd00ab066f");
	let serial = get_string_field(&table, |t| t.serial_number(), "Default string");
	let family = get_string_field(&table, |t| t.family(), "X570 MB");

	map.add_fields(
		SmBiosType::SystemInformation,
		vec![
			("version", version),
			("sku", sku),
			("product", product),
			("manufacturer", manufacturer),
			("uuid", uuid),
			("serial", serial),
			("family", family),
		],
	);
}

fn populate_baseboard_information(map: &mut SmBiosMap, table: Option<SMBiosBaseboardInformation>) {
	let asset = get_string_field(&table, |t| t.asset_tag(), "Default string");
	let version = get_string_field(&table, |t| t.version(), "Default string");
	let product = get_string_field(&table, |t| t.product(), "X570 AORUS ULTRA");
	let location = get_string_field(&table, |t| t.location_in_chassis(), "Default string");
	let manufacturer = get_string_field(&table, |t| t.manufacturer(), "Gigabyte Technology Co., Ltd.");
	let serial = get_string_field(&table, |t| t.serial_number(), "Default string");

	map.add_fields(
		SmBiosType::BaseboardInformation,
		vec![
			("asset", asset),
			("version", version),
			("product", product),
			("location", location),
			("manufacturer", manufacturer),
			("serial", serial),
		],
	);
}

fn populate_enclosure_information(map: &mut SmBiosMap) {
	map.add_fields(
		SmBiosType::EnclosureInformation,
		vec![
			("asset", "Default string"),
			("version", "Default string"),
			("sku", "Default string"),
			("manufacturer", "Default string"),
			("serial", "Default string"),
		],
	);
}

fn populate_processor_information(map: &mut SmBiosMap, table: Option<SMBiosProcessorInformation>) {
	let asset = get_string_field(&table, |t| t.asset_tag(), "Unknown");
	let version = get_string_field(&table, |t| t.processor_version(), "AMD Ryzen 9 5950X 16-Core Processor");
	let part = get_string_field(&table, |t| t.part_number(), "Zen");
	let manufacturer = get_string_field(&table, |t| t.processor_manufacturer(), "Advanced Micro Devices, Inc.");
	let serial = get_string_field(&table, |t| t.serial_number(), "Unknown");
	let sock_pfx = get_string_field(&table, |t| t.socket_designation(), "AM4");

	map.add_fields(
		SmBiosType::ProcessorInformation,
		vec![
			("asset", asset),
			("version", version),
			("part", part),
			("manufacturer", manufacturer),
			("serial", serial),
			("sock_pfx", sock_pfx),
		],
	);
}

fn populate_oem_strings(map: &mut SmBiosMap) {
	map.add_field(SmBiosType::OemStrings, "value", "Default string");
}

fn populate_memory_device(map: &mut SmBiosMap, table: Option<SMBiosMemoryDevice>) {
	let bank = get_string_field(&table, |t| t.bank_locator(), "Bank 0");
	let asset = get_string_field(&table, |t| t.asset_tag(), "Not Specified");
	let part = get_string_field(&table, |t| t.part_number(), "OV_8GR1");
	let manufacturer = get_string_field(&table, |t| t.manufacturer(), "OEM_VENDOR");
	let serial = get_string_field(&table, |t| t.serial_number(), "OEM33162");
	let loc_pfx = get_string_field(&table, |t| t.device_locator(), "DIMM 0");

	map.add_fields(
		SmBiosType::MemoryDevice,
		vec![
			("bank", bank),
			("asset", asset),
			("part", part),
			("manufacturer", manufacturer),
			("speed", String::from("3200")),
			("serial", serial),
			("loc_pfx", loc_pfx),
		],
	);
}

fn get_table<'a, T: SMBiosStruct<'a>>(dmi: &'a SMBiosData) -> Option<T> {
	let table = dmi.first::<T>();

	if table.is_none() {
		log::warn!(
			"unable to get table for {}, will use defaults for every field",
			type_name::<T>()
		)
	}

	table
}

fn get_field<T, R>(table: &Option<T>, f: impl FnOnce(&T) -> Option<R>, default: impl Into<R>) -> R {
	let Some(table) = table else {
		return default.into();
	};

	f(table).unwrap_or_else(|| default.into())
}

fn get_string_field<T>(table: &Option<T>, f: impl FnOnce(&T) -> SMBiosString, default: impl Into<String>) -> String {
	let Some(table) = table else {
		return default.into();
	};

	f(table).to_string()
}

fn get_uuid_field<T>(
	table: &Option<T>,
	f: impl FnOnce(&T) -> Option<SystemUuidData>,
	default: impl Into<String>,
) -> String {
	if let Some(table) = table {
		if let Some(SystemUuidData::Uuid(uuid)) = f(table) {
			return uuid.to_string();
		}
	}

	default.into()
}
