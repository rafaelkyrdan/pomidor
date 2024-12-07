use cursive::{CbSink, Cursive, CursiveExt};
use cursive::views::{Dialog, TextView};
use cursive::view::Nameable;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() {

    let mut siv:Cursive = Cursive::default();

    let pomodoro_duration = Arc::new(Mutex::new(25*60));
    let break_duration = Arc::new(Mutex::new(5*60));
    let timer_counter = Arc::new(Mutex::new(*pomodoro_duration.lock().unwrap()));

    let is_break_time = Arc::new(Mutex::new(false));
    let is_running = Arc::new(Mutex::new(false));

    let timer_counter_clone = Arc::clone(&timer_counter);
    let is_break_time_clone = Arc::clone(&is_break_time);
    let is_running_clone = Arc::clone(&is_running);

    let text_view = TextView::new(format!(
        "Time: {}", format_time(*timer_counter.lock().unwrap())
    )).with_name("timer");

    siv.add_layer(
        Dialog::around(text_view)
        .button("+1 min", create_time_adjust_callback(
            Arc::clone(&pomodoro_duration),
            Arc::clone(&timer_counter),
            60
        ))
        .button("-1 min", create_time_adjust_callback(
            Arc::clone(&pomodoro_duration),
            Arc::clone(&timer_counter),
            -60
        ))
        .button("Start/Stop", create_start_stop_callback(
            Arc::clone(&is_running)
        ))
        .button("Quit", |s| s.quit())
        .title("Pomodoro timer")
    );

    create_timer_thread(
        Arc::clone(&timer_counter_clone),
        Arc::clone(&is_break_time_clone),
        Arc::clone(&is_running_clone),
        Arc::clone(&pomodoro_duration),
        Arc::clone(&break_duration)
    );

    let cb_sink = siv.cb_sink().clone();
    let timer_counter_for_refresh = Arc::clone(&timer_counter);


    create_refresh_thread(
        Arc::clone(&timer_counter_for_refresh),
        cb_sink.clone()
    );


    siv.run();


}

fn create_timer_thread(
    timer_counter: Arc<Mutex<usize>>,
    is_break_time: Arc<Mutex<bool>>,
    is_running: Arc<Mutex<bool>>,
    pomodoro_duration: Arc<Mutex<usize>>,
    break_duration: Arc<Mutex<usize>>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(1));
            
            // Using a block to ensure locks are dropped as soon as possible
            {
                let mut time_left = timer_counter.lock().unwrap();
                let mut break_time = is_break_time.lock().unwrap();
                let running = is_running.lock().unwrap();

                if *running {
                    if *time_left > 0 {
                        *time_left -= 1;
                    } else {
                        if *break_time {
                            *time_left = *pomodoro_duration.lock().unwrap();
                            *break_time = false;
                            println!("Back to work!");
                        } else {
                            *time_left = *break_duration.lock().unwrap();
                            *break_time = true;
                            println!("Take a break!");
                        }
                    }
                }
            }
        }
    })
}

fn create_refresh_thread(
    timer_counter: Arc<Mutex<usize>>,
    cb_sink: CbSink,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(1));
            let timer_counter_refresh = Arc::clone(&timer_counter);
            
            cb_sink.send(Box::new(move |s| {
                let time_left = *timer_counter_refresh.lock().unwrap();
                s.call_on_name("timer", |view: &mut TextView| {
                    view.set_content(format!("Time: {}", format_time(time_left)));
                });
            })).unwrap();
        }
    })
}

fn format_time(time: usize) ->  String {
    let minutes = time / 60;
    let seconds = time % 60;
    format!("{:02}:{:02}", minutes, seconds)
}

fn create_time_adjust_callback(
    pomodoro_duration: Arc<Mutex<usize>>,
    timer_counter: Arc<Mutex<usize>>,
    adjustment: i32,
) -> impl Fn(&mut Cursive) {
    move |s| {
        let mut duration = pomodoro_duration.lock().unwrap();
                
        if adjustment > 0 || *duration > 60 {
            *duration = (*duration as i32 + adjustment) as usize;
            
            let mut timer_value = timer_counter.lock().unwrap();
            *timer_value = *duration;
            
            s.call_on_name("timer", |view: &mut TextView| {
                view.set_content(format!("Time: {}", format_time(*duration)));
            });
        }
    }
}

fn create_start_stop_callback(
    is_running: Arc<Mutex<bool>>,
) -> impl Fn(&mut Cursive) {
    move |s| {
        let mut running = is_running.lock().unwrap();
        *running = !*running;
        
        s.call_on_name("timer", |view: &mut TextView| {
            view.set_content(
                if *running { "Timer is running" } else { "Timer is paused" }
            );
        });
    }
}