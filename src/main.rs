use gtk::gdk::Display;
use gtk::gio::ActionEntry;
use gtk::glib::{self, clone};
use gtk::prelude::*;
use gtk::CssProvider;

const WIDTH: i32 = 600;
const HEIGHT: i32 = 192;

fn main() {
    let application = gtk::Application::new(Some("com.naughie.clock"), Default::default());
    application.connect_startup(|_| load_css());
    application.connect_activate(build_ui);
    application.run();
}

fn load_css() {
    // Load the CSS file and add it to the provider
    let provider = CssProvider::new();
    provider.load_from_data(include_str!("style.css"));

    // Add the provider to the default screen
    gtk::style_context_add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn current_time() -> String {
    use chrono::Timelike;

    let now = chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(9 * 3600).unwrap());
    let time = now.time();
    let i_hour = time.hour();
    let i_hour = if i_hour == 0 { 24 } else { i_hour };
    let i_minute = time.minute();
    format!("{i_hour:02}{i_minute:02}")
}

struct ClockUpdater {
    hour: gtk::Label,
    minute: gtk::Label,
}

fn clock_body() -> (gtk::Box, ClockUpdater) {
    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);

    let now = current_time();

    let hour = gtk::Label::new(None);
    hour.set_text(&now[..2]);

    let delim = gtk::Label::new(None);
    delim.set_text(":");
    delim.set_margin_bottom(25);
    delim.set_margin_start(3);
    delim.set_margin_end(3);

    let minute = gtk::Label::new(None);
    minute.set_text(&now[2..]);

    hbox.set_css_classes(&["clock"]);
    hbox.set_halign(gtk::Align::Center);

    hbox.append(&hour);
    hbox.append(&delim);
    hbox.append(&minute);

    (hbox, ClockUpdater { hour, minute })
}

fn build_ui(application: &gtk::Application) {
    let titlebar = gtk::Label::new(Some("Clock"));
    let headerbar = gtk::HeaderBar::new();
    headerbar.set_show_title_buttons(true);
    headerbar.set_title_widget(Some(&titlebar));

    let window = gtk::ApplicationWindow::new(application);

    // window.set_title(Some("Clock"));
    window.set_titlebar(Some(&headerbar));
    window.set_default_size(WIDTH, HEIGHT);

    let (label, updater) = clock_body();

    let action_update = ActionEntry::builder("update")
        .parameter_type(None)
        .activate(move |_, _, _| {
            let now = current_time();
            updater.hour.set_text(&now[..2]);
            updater.minute.set_text(&now[2..]);
        })
        .build();

    window.set_child(Some(&label));
    window.add_action_entries([action_update]);

    let (sender, receiver) = async_channel::bounded(1);
    gtk::gio::spawn_blocking(move || {
        use std::time::Duration;
        loop {
            std::thread::sleep(Duration::from_secs(5));
            if sender.send_blocking(()).is_err() {
                break;
            }
        }
    });

    gtk::glib::spawn_future_local(clone!(
        #[weak]
        window,
        async move {
            while receiver.recv().await.is_ok() {
                <_ as ActionGroupExt>::activate_action(&window, "update", None);
            }
        }
    ));

    window.show();
}
