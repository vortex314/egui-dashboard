let mut db_clone = dashboard.clone();
let mut db_clone2 = dashboard.clone();

let mut dashboard_message_handler = MessageHandler::new(dashboard.clone());
pubsub.add_listener(dashboard_message_handler.sink_ref());

let mut file_change = FileChange::new(args.config.clone());
let pubsub_sink_ref = pubsub.sink_ref();
file_change.for_all(Box::new(move |fc: FileChangeEvent| {
    match fc {
        FileChangeEvent::FileChange(file_name) => {
            info!("File changed {}", file_name);
            let mut error_config = Tag::new("label".to_string());
            error_config.label = Some("Error loading config file".to_string());
            let mut dashboard_config = Box::new(load_xml_file(&file_name).or(Some(error_config)).unwrap());
            let dashboard_title = dashboard_config
                .label
                .clone()
                .unwrap_or(String::from("Dashboard"));
            db_clone2
                .lock()
                .unwrap()
                .load(&mut dashboard_config)
                .unwrap();
        }
    }
}));