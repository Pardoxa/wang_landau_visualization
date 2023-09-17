use egui::{
    emath::Align,
    Layout, 
    plot::*,
    Visuals,
    Button
};
use sampling::*;
use std::{time::{Instant, Duration}, thread};
use sampling::norm_log10_sum_to_1;
use crate::{CoinSeq, generate_cs};


pub struct SimData{
    c: CoinSeq
}

#[derive(PartialEq, Eq)]
pub enum Scale{
    Log,
    Lin
}
#[derive(PartialEq)]
pub enum LightMode{
    Light,
    Dark
}


pub struct AppState{
    sim: Option<SimData>,
    pause: bool,
    log_scale: bool,
    speed: f64,
    n: usize,
    log_f: Vec<[f64;2]>,
    start_time: Option<Instant>,
    pause_time: Option<Instant>,
    pause_duration: Duration,
    log_f_logscale: bool,
    seed: u64,
    step_size: usize,
    pixel: f32,
    linewidth: f32,
    threshold: f64,
    refine_steps: usize,
    hist_scale: Scale,
    l_mode: LightMode
}

impl Default for AppState{
    fn default() -> Self {
        Self { 
            sim: None, 
            pause: false, 
            log_scale: true, 
            speed: 1.2, 
            n: 1500, 
            log_f: Vec::new(), 
            start_time: None, 
            log_f_logscale: false,
            step_size: 1,
            seed: 834628956578,
            pixel: 2.0,
            linewidth: 1.5,
            threshold: 0.000001,
            pause_time: None,
            pause_duration: Duration::new(0, 0),
            refine_steps: 10000000,
            hist_scale: Scale::Lin,
            l_mode: LightMode::Light
        }
    }
}

impl AppState{
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        //if let Some(storage) = cc.storage {
        //    return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        //}
        cc.egui_ctx.set_visuals(Visuals::light());
        cc.egui_ctx.set_pixels_per_point(2.0);
        Default::default()
    }
}


impl eframe::App for AppState {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        //eframe::set_value(storage, eframe::APP_KEY, self);

        // DO NOT SAVE
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            pause,
            sim,
            log_scale,
            speed,
            n,
            log_f,
            start_time,
            log_f_logscale,
            seed,
            step_size,
            pixel,
            linewidth,
            threshold,
            pause_duration,
            pause_time,
            refine_steps,
            hist_scale,
            l_mode
        } = self;
        //// Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui


        egui::SidePanel::left("side_panel")
            .default_width(300.0)
            .show(ctx, |ui| {


            egui::ScrollArea::both().show(
                ui,
                |ui|
                {

                    match l_mode{
                        LightMode::Dark => {
                            if ui.add(
                                 Button::new("☀").frame(false)
                             ).on_hover_text("Wechsel in den hellen Modus")
                             .clicked(){
                                ui.ctx().set_visuals(Visuals::light());
                                *l_mode = LightMode::Light;
                            }
                        },
                        LightMode::Light => {
                            if ui.add(
                                 Button::new("🌙").frame(false)
                             ).on_hover_text("Wechsel in den Dunklen Modus")
                             .clicked(){
                                ui.ctx().set_visuals(Visuals::dark());
                                *l_mode = LightMode::Dark;
                            }
                        }
                    }

                    if ui.add(egui::Button::new("Start"))
                        .on_hover_text("Startet die Simulation.")
                        .clicked()
                    {
                        *sim = Some(
                            SimData { c: generate_cs(*n, *seed, *step_size, *threshold) }
                        );
                        *log_f = Vec::new();
                        *start_time = Some(Instant::now());
                        *pause_duration = Duration::new(0, 0);
                        if let Some(ins) = pause_time
                        {
                            *ins = Instant::now();
                        }
                    }
                    let btn_text = if *pause{
                        "Fortfahren"
                    } else{
                        "Pausieren"
                    };
                    if ui.add(egui::Button::new(btn_text))
                        .clicked()
                    {
                        *pause = !*pause;
                        if *pause {
                            *pause_time = Some(Instant::now());
                        } else {
                            let dur = pause_time.as_ref().unwrap().elapsed();
                            *pause_time = None;
                            *pause_duration += dur;
                        }
                    }

                    let btn_text = if *log_scale {
                        "Switch to linear Scale"
                    } else {
                        "Switch to logscale"
                    };

                    if ui.add(egui::Button::new(btn_text))
                        .clicked()
                    {
                        *log_scale = !*log_scale;
                    }
                    ui.add(egui::Slider::new(speed, 0.0..=10.0).logarithmic(false).text("Speed"));
                    ui.add(egui::Slider::new(n, 10..=10000).logarithmic(true).text("N"));
                    ui.add(egui::Slider::new(seed, 0..=u64::MAX).logarithmic(true).text("Seed"));
                    ui.add(egui::Slider::new(step_size, 1..=30).logarithmic(false).text("step size"));

                    let btn_text = if *log_f_logscale {
                        "to_log_f"
                    } else {
                        "to log_log_f"
                    };

                    if ui.add(egui::Button::new(btn_text))
                        .clicked()
                    {
                        *log_f_logscale = !*log_f_logscale;
                    }

                    ui.add(egui::Slider::new(pixel, 1.0..=5.0).logarithmic(false).text("Zoom"));
                    if ui.add(egui::Button::new("Rescale"))
                        .clicked()
                    {
                        ctx.set_pixels_per_point(*pixel);
                    }

                    ui.add(egui::Slider::new(linewidth, 0.0..=10.0).logarithmic(false).text("line"));
                    ui.add(egui::Slider::new(threshold, 0.00000000001..=0.001).logarithmic(true).text("threshold"));
                    ui.add(egui::Slider::new(refine_steps, 0..=usize::MAX).logarithmic(true).text("E refine"));
                    ui.radio_value(hist_scale, Scale::Lin, "Hist Lin");
                    ui.radio_value(hist_scale, Scale::Log, "Hist Log");
                }
            );
            
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            //ui.heading("Random Walker");
            //ui.hyperlink("https://github.com/emilk/eframe_template");
            //ui.add(egui::github_link_file!(
            //    "https://github.com/emilk/eframe_template/blob/master/",
            //    "Source code."
            //));

            if let Some(sim_data) = sim{

                let time = Instant::now();
                if !*pause
                {
                    let s = *speed;
                    let wl = sim_data.c.wl.clone();
                    let t = thread::spawn(
                        move || {
                            wl.write().unwrap().wang_landau_while_acc(
                                |ensemble, step, old_energy| {
                                    ensemble.update_head_count(step, old_energy)
                                }, 
                                |_| {(time.elapsed().as_millis() as f64) < (30.0_f64 * s)}
                            );
                        }
                    );

                    sim_data.c.entr.entropic_sampling_while_acc(
                        |ensemble, step, old_energy| {
                            ensemble.update_head_count(step, old_energy)
                        }, 
                        |_| {}, 
                        |_| {(time.elapsed().as_millis() as f64) < (30.0_f64 * s)}
                    );

                    if sim_data.c.entr.step_counter() > *refine_steps{
                        sim_data.c.entr.refine_estimate();
                    }

                    t.join().unwrap();
                }

                if !*pause && ! sim_data.c.wl.read().unwrap().is_finished() {
                    let current_log_f: f64 = sim_data.c.wl.read().unwrap().log_f();
                    let ellased = start_time.as_ref().unwrap().elapsed() - *pause_duration;
                    log_f.push([ellased.as_secs_f64(), current_log_f]);
                }
                

                let layout = Layout{
                    main_dir: egui::Direction::LeftToRight,
                    main_wrap: false,
                    main_align: Align::Center,
                    main_justify: true,
                    cross_align: Align::Min,
                    cross_justify: true
                };

               ui.with_layout(
                        layout, 
                        |ui|{
                            let max_width = ui.available_width();
                            let mut density = sim_data.c.wl.read().unwrap().log_density_base10();
                            let mut true_density = sim_data.c.log_prob_true.clone();
                            let mut e_data = sim_data.c.entr.log_density_base10();
                            norm_log10_sum_to_1(&mut e_data);
                            norm_log10_sum_to_1(&mut density);
                            norm_log10_sum_to_1(&mut true_density);
                            if !*log_scale
                            {
                                density.iter_mut()
                                    .for_each(|val| *val = 10.0f64.powf(*val));
                                true_density.iter_mut()
                                    .for_each(|val| *val = 10.0f64.powf(*val));
                                e_data.iter_mut()
                                    .for_each(|val| *val = 10.0f64.powf(*val));
                            }
                            let len = density.len();
    
                            let wl_density: Vec<_> = 
                                density.into_iter()
                                    .enumerate()
                                .map(
                                    |(idx, den)|
                                    {
                                        let x = idx as f64 / len as f64;
                                        let y = den;
                                        [x,y]
                                    }
                                ).collect();
    
                            let true_density: Vec<_> = 
                                true_density.into_iter()
                                    .enumerate()
                                .map(
                                    |(idx, den)|
                                    {
                                        let x = idx as f64 / len as f64;
                                        let y = den;
                                        [x,y]
                                    }
                                ).collect();
                            let e_density: Vec<_> = 
                                e_data.into_iter()
                                    .enumerate()
                                .map(
                                    |(idx, den)|
                                    {
                                        let x = idx as f64 / len as f64;
                                        let y = den;
                                        [x,y]
                                    }
                                ).collect();
                            ui.vertical(
                                |ui|
                                {
    
                                    let hight = ui.available_height();
                                    Plot::new("plot_average_etc")
                                    .include_x(0.0)
                                    .legend(Legend::default())
                                    .height(hight - 25.0)
                                    .width(max_width * 0.5)
                                    .show(
                                        ui, 
                                        |plot_ui|
                                        {
                                            
                                            let true_line = Line::new(true_density).name("analytic Results").width(*linewidth*2.0);
                                            let wl_line = Line::new(wl_density).name("WL Results").width(*linewidth);
                                            
                                            
                                            let ent_line = Line::new(e_density).name("Entropic Results").width(*linewidth);
                                            
                                            plot_ui.line(true_line);
                                            plot_ui.line(wl_line);
                                            plot_ui.line(ent_line);
                                            
                                            //let y = plot_ui.plot_bounds().max()[1];
                                            //let x = plot_ui.plot_bounds().max()[0];
                                            //
                                            //let text = egui::plot::Text::new(PlotPoint { x: x / 20.0, y: y / 2.0 }, "d")
                                            //    .anchor(Align2::LEFT_CENTER);
                                            //plot_ui.text(text);
                                        }
                                    );
                                    ui.label("heads rate");
                                }
                            );
                            ui.vertical(
                                |ui|
                                {
                                    let name = if *log_f_logscale{
                                        "log10(logE(f))"
                                    } else {
                                        "logE(f)"
                                    };
                                    let hight = ui.available_height();
                                    Plot::new("plot_log_f")
                                    .include_x(0.0)
                                    .include_y(0.0)
                                    .auto_bounds_y()
                                    .legend(Legend::default())
                                    .height((hight - 25.0)*0.5)
                                    .show(
                                        ui, 
                                        |plot_ui|
                                        {
                                            let mut tmp_log_f = log_f.clone();
                                            if *log_f_logscale
                                            {
                                                tmp_log_f.iter_mut()
                                                    .for_each(|[_, val]| *val = val.log10());
                                            }
                                            
                                            let log_f_line = Line::new(tmp_log_f).name(name)
                                            .width(*linewidth);
                                            
    
                                            plot_ui.line(log_f_line);
                                            
                                        }
                                    );
                                    ui.label(name);
                                    let mut hist: Vec<_> = sim_data.c.wl.read().unwrap().hist().bin_hits_iter()
                                        .map(|(bin, hits)| [bin as f64 / len as f64, hits as f64])
                                        .collect();
                                    let mut ent_hist: Vec<_> = sim_data.c.entr.hist().bin_hits_iter()
                                        .map(|(bin, hits)| [bin as f64 / len as f64, hits as f64])
                                        .collect();

                                    if matches!(*hist_scale, Scale::Log) {
                                        hist.iter_mut()
                                            .for_each(
                                                |[_, val]|
                                                {
                                                    if *val < 1.0 {
                                                        *val = f64::NAN;   
                                                    } else {
                                                        *val = val.log10();
                                                    }
                                                    
                                                }
                                            );
                                        ent_hist.iter_mut()
                                            .for_each(
                                                |[_, val]|
                                                {
                                                    if *val < 1.0 {
                                                        *val = f64::NAN;   
                                                    } else {
                                                        *val = val.log10();
                                                    }
                                                }
                                            );
                                    }

                                    let hight = ui.available_height();
                                    Plot::new("plot_histogram")
                                    .include_x(0.0)
                                    .include_y(0.0)
                                    .auto_bounds_y()
                                    .legend(Legend::default())
                                    .height(hight - 25.0)
                                    .show(
                                        ui, 
                                        |plot_ui|
                                        {
                                            
                                            let histogram = Line::new(hist).name("Wang Landau Histogram")
                                                .width(*linewidth);
                                            
                                            let ent_line = Line::new(ent_hist).name("Entropic Histogram")
                                                .width(*linewidth);
    
                                            plot_ui.line(histogram);
                                            plot_ui.line(ent_line);
                                            
                                        }
                                    );
                                    ui.label("histogram");
                                    ctx.request_repaint();
                                }
                            );
                        }
                    );
            }
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally choose either panels OR windows.");
            });
        }
        
    }
}


