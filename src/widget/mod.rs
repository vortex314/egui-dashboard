trait Widget {
    fn draw(&self, ui: &mut Ui) -> Result<(), String>;
    fn on_message(&mut self, message: &str);
}