use gtk::builders::ConstraintBuilder;
use gtk::gio::ffi::GOutputVector;

use rusqlite::{Connection, Result};

use gtk::ffi::gtk_grid_remove_row;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Button, Label, Box, Orientation, Entry, CssProvider, StyleContext, Align, Grid};
use gtk::gdk;

use std::sync::{Arc, Mutex};
    
fn main() -> Result<()> {
    let app = Application::new(Some("com.example.RustySnout"), Default::default());
    
    let conn = Connection::open("data.db")?;
   let mut stmt = conn.prepare("SELECT process_name, time, block_number FROM App WHERE block_number = (SELECT MAX(block_number) FROM App)")?;
let mut stmt2 = conn.prepare("SELECT pid, process_name, up_bps, down_bps, connections, time, block_number FROM processes WHERE block_number = (SELECT MAX(block_number) FROM processes)")?;
let mut stmt3 = conn.prepare("SELECT cid, source, destination, protocol, up_bps, down_bps, process_name, time, block_number FROM connections WHERE block_number = (SELECT MAX(block_number) FROM connections)")?;

    let mut data1: Vec<Vec<String>> = Vec::new();
    let mut data2: Vec<Vec<String>> = Vec::new();
    let mut data3: Vec<Vec<String>> = Vec::new();    

    let rows = stmt.query_map([], |row| {
        Ok(vec![
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)?.to_string(),
            row.get::<_, i64>(2)?.to_string(),
        ])
    })?;
    for row in rows {
        data1.push(row?);
    }

    let rows2 = stmt2.query_map([], |row| {
        Ok(vec![
            row.get::<_, i64>(0)?.to_string(),
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?.to_string(),
            row.get::<_, i64>(3)?.to_string(),
            row.get::<_, i64>(4)?.to_string(),
            row.get::<_, i64>(5)?.to_string(),
            row.get::<_, i64>(6)?.to_string(),
        ])
    })?;
    
    for row in rows2 {
        data2.push(row?);
    }

    let rows3 = stmt3.query_map([], |row| {
        Ok(vec![
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, i64>(4)?.to_string(),
            row.get::<_, i64>(5)?.to_string(),
            row.get::<_, String>(6)?,
            row.get::<_, i64>(7)?.to_string(),
            row.get::<_, i64>(8)?.to_string(),
        ])
    })?;

    for row in rows3 {
        data3.push(row?);      
    }

    app.connect_activate(move |app| {
       let shared_content1 = Arc::new(Mutex::new(data1.clone()));
       let shared_content2 = Arc::new(Mutex::new(data2.clone()));
       let shared_content3 = Arc::new(Mutex::new(data3.clone()));        
        build_ui(app, shared_content1.clone(),shared_content2.clone(),shared_content3.clone());
    });

    app.run();

    Ok(())
}

fn build_ui(app: &Application, shared_content1: Arc<Mutex<Vec<Vec<String>>>>,shared_content2: Arc<Mutex<Vec<Vec<String>>>>,shared_content3: Arc<Mutex<Vec<Vec<String>>>>) {
    let provider = CssProvider::new();
    provider.load_from_data(b"* { font-family: monospace; background-color: #1e1e1e; color: #ffffff; } \
                #HeaderLabel { color: #00ff00; } \
                button { margin: 5px; } \
                grid { border: 1px solid #ccc; padding: 10px; }"); 

    StyleContext::add_provider_for_display(&gdk::Display::default().unwrap(), &provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

    let setup_window = ApplicationWindow::builder().application(app).title("Setup - Refresh Rate").default_width(350).default_height(100).build();
    let refresh_rate_entry = Entry::builder().placeholder_text("Enter refresh rate in seconds (Min: 0.5)").build();
    let next_button = Button::builder().label("Next").build();
    let setup_content = Box::new(Orientation::Vertical, 5);
    setup_content.append(&refresh_rate_entry);
    setup_content.append(&next_button);
    setup_window.set_child(Some(&setup_content));
    setup_window.show();

    let main_window = ApplicationWindow::builder().application(app).title("RustySnout - Main Window").default_width(1000).default_height(600).build();
    let content = Box::new(Orientation::Vertical, 5);
    let header_label = Label::builder().label("--------------------").name("HeaderLabel").build();
    let display_label = Label::builder().label("Select a button to display its corresponding usage data.").build();
    let data_grid = Grid::new();
    data_grid.set_row_spacing(10);
    data_grid.set_column_spacing(10);
    data_grid.set_row_homogeneous(true);
    data_grid.set_column_homogeneous(true);
    let button1 = Button::builder().label("Process").build();
    let button2 = Button::builder().label("Connection").build();
    let button3 = Button::builder().label("Remote-Address").build();
    let buttons_box = Box::new(Orientation::Horizontal, 0);
    buttons_box.set_valign(Align::Center);
    buttons_box.set_halign(Align::Center);
    buttons_box.append(&button1);
    buttons_box.append(&button2);
    buttons_box.append(&button3);
    content.append(&header_label);
    content.append(&display_label);
    content.append(&buttons_box);
    content.append(&data_grid);

    let process_headers = vec!["Process Name", "Time", "Block Number"];
let connection_headers = vec!["PID", "Process Name", "Upload Bps", "Download Bps", "Connections", "Time", "Block Number"];
let remote_address_headers = vec![ "Source", "Destination", "Protocol", "Upload Bps", "Download Bps", "Process Name", "Time", "Block Number"];

    main_window.set_child(Some(&content));

    next_button.connect_clicked(move |_| {
        let refresh_rate = refresh_rate_entry.text().parse::<f64>().unwrap_or(0.5).max(0.5);
        header_label.set_text(&format!(" Refresh Rate: {}s", refresh_rate));
        setup_window.hide();
        main_window.show();
    });

button1.connect_clicked({
    let display_label = display_label.clone();
    let data_grid = data_grid.clone();
    let content1 = shared_content1.clone();
    let process_headers = process_headers.clone(); // Assuming headers are defined outside
    move |_| {
        display_label.set_label("Displaying usage data by: Process");

         // Properly remove all widgets from the grid
        let mut child = data_grid.first_child();
        while let Some(widget) = child {
            let next = widget.next_sibling(); // Get the next sibling to iterate before removing the current child
            data_grid.remove(&widget); // Remove each child widget
            child = next; // Move to the next child
        }
        
        for (j, header) in process_headers.iter().enumerate() {
            let header_label = Label::builder().label(header).build();
            data_grid.attach(&header_label, j as i32, 0, 1, 1); // Attach headers at the first row
        }
        let data = content1.lock().unwrap();
        for (i, row) in data.iter().enumerate() {
            for (j, value) in row.iter().enumerate() {
                let label = Label::new(Some(value));
                data_grid.attach(&label, j as i32, (i as i32) + 1, 1, 1); // Offset by one row for headers
            }
        }
    }
});

button2.connect_clicked({
    let display_label = display_label.clone();
    let data_grid = data_grid.clone();
    let content2 = shared_content2.clone();
    let connection_headers = connection_headers.clone(); // Assuming headers are defined outside
    move |_| {
        display_label.set_label("Displaying usage data by: Connection");

        // Properly remove all widgets from the grid
        let mut child = data_grid.first_child();
        while let Some(widget) = child {
            let next = widget.next_sibling(); // Get the next sibling to iterate before removing the current child
            data_grid.remove(&widget); // Remove each child widget
            child = next; // Move to the next child
        }

        for (j, header) in connection_headers.iter().enumerate() {
            let header_label = Label::builder().label(header).build();
            data_grid.attach(&header_label, j as i32, 0, 1, 1); // Attach headers at the first row
        }
        let data = content2.lock().unwrap();
        for (i, row) in data.iter().enumerate() {
            for (j, value) in row.iter().enumerate() {
                let label = Label::new(Some(value));
                data_grid.attach(&label, j as i32, (i as i32) + 1, 1, 1); // Offset by one row for headers
            }
        }
    }
});


button3.connect_clicked({
    let display_label = display_label.clone();
    let data_grid = data_grid.clone();
    let content3 = shared_content3.clone();
    let remote_address_headers = remote_address_headers.clone(); // Ensure headers are cloned or accessible in the closure
    move |_| {
        display_label.set_label("Displaying usage data by: Remote-Address");

        // Properly remove all widgets from the grid
        let mut child = data_grid.first_child();
        while let Some(widget) = child {
            let next = widget.next_sibling(); // Get the next sibling to iterate before removing the current child
            data_grid.remove(&widget); // Remove each child widget
            child = next; // Move to the next child
        }

        // Attach new headers at the first row
        for (j, header) in remote_address_headers.iter().enumerate() {
            let header_label = gtk::Label::builder().label(header).build();
            data_grid.attach(&header_label, j as i32, 0, 1, 1);
        }

        // Retrieve data and display it, offset by one row for headers
        let data = content3.lock().unwrap();
        for (i, row) in data.iter().enumerate() {
            for (j, value) in row.iter().enumerate() {
                let label = gtk::Label::new(Some(value));
                data_grid.attach(&label, j as i32, (i as i32) + 1, 1, 1); // Offset by one row for headers
            }
        }
    }
});

}
