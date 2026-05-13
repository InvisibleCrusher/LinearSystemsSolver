#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Linear System Solver",
        options,
        Box::new(|cc| {
            //set zoom after creating the egui context
            cc.egui_ctx.set_zoom_factor(1.5);
            Ok(Box::<MyApp>::default())
        }),
    )
}

struct MyApp {
    matrix: Vec<Vec<String>>,
    results: Vec<String>,
    cleanedmatrix: Vec<Vec<f64>>,
    cleanedresults: Vec<f64>,
    solution: Option<Vec<f64>>,
    error: Option<String>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            matrix: vec![vec!["".to_string(); 3]; 3],
            results: vec!["".to_string(); 3],
            cleanedmatrix: vec![vec![0.0; 3]; 3],
            cleanedresults: vec![0.0; 3],
            solution: None,
            error: None,
        }
    }
}

impl MyApp {
    pub fn set_size(&mut self, new_size: usize) {
        self.matrix.resize(new_size, vec!["".to_string(); new_size]);
        for row in &mut self.matrix {
            row.resize(new_size, "".to_string());
        }

        self.results.resize(new_size, "".to_string());

        self.cleanedmatrix.resize(new_size, vec![0.0; new_size]);
        for row in &mut self.cleanedmatrix {
            row.resize(new_size, 0.0);
        }
        self.cleanedresults.resize(new_size, 0.0);

        //reset
        self.solution = None;
        self.error = None;
    }
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.button("➖").clicked() && self.matrix.len() > 2 {
                    let new_size = self.matrix.len() - 1;
                    self.set_size(new_size);
                }
                if ui.button("➕").clicked() && self.matrix.len() < 10 {
                    let new_size = self.matrix.len() + 1;
                    self.set_size(new_size);
                }
                if ui.button("Clear").clicked() {
                    for i in 0..self.matrix.len() {
                        for j in 0..self.matrix[i].len() {
                            self.matrix[i][j].clear();
                        }
                        self.results[i].clear();
                    }
                    self.solution = None;
                    self.error = None;
                }
            });

            let box_size = egui::vec2(60.0, 24.0);
            let labels = ["x", "y", "z", "w", "a", "b", "c", "d", "e", "f"];

            let grid_spacing = ui.spacing().item_spacing / 2.0;

            egui::Grid::new("inputgrid")
                .spacing(grid_spacing)
                .show(ui, |ui| {
                    //header row
                    for j in 0..self.matrix.len() {
                        let label_text = labels
                            .get(j)
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| format!("x{}", j));

                        ui.allocate_ui_with_layout(
                            box_size,
                            egui::Layout::top_down(egui::Align::Center),
                            |ui| {
                                ui.label(label_text);
                            },
                        );
                    }
                    ui.label("");
                    ui.end_row();

                    //data rows
                    for i in 0..self.matrix.len() {
                        for j in 0..self.matrix[i].len() {
                            ui.allocate_ui_with_layout(
                                box_size,
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    ui.add(
                                        egui::TextEdit::singleline(&mut self.matrix[i][j])
                                            .desired_width(box_size.x),
                                    );
                                },
                            );
                        }

                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            ui.label("=");
                            ui.allocate_ui_with_layout(
                                box_size,
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    ui.add(
                                        egui::TextEdit::singleline(&mut self.results[i])
                                            .desired_width(box_size.x),
                                    );
                                },
                            );
                        });

                        ui.end_row();
                    }
                });
            ui.add_space(10.0);

            if ui.button("Calculate").clicked() {
                self.error = None;
                self.solution = None;

                //parse matrix
                for i in 0..self.matrix.len() {
                    for j in 0..self.matrix[i].len() {
                        if let Ok(val) = self.matrix[i][j].parse::<f64>() {
                            self.cleanedmatrix[i][j] = val;
                        } else {
                            self.error = Some(format!("error in row {}, column {}", i + 1, j + 1));
                        }
                    }
                }

                //parse results
                if self.error.is_none() {
                    for i in 0..self.results.len() {
                        if let Ok(val) = self.results[i].parse::<f64>() {
                            self.cleanedresults[i] = val;
                        } else {
                            self.error = Some(format!("error in result for row {}", i + 1));
                        }
                    }
                }

                //solve
                if self.error.is_none() {
                    if let Some(res) =
                        solve(self.cleanedmatrix.clone(), self.cleanedresults.clone())
                    {
                        self.solution = Some(res);
                    } else {
                        self.error =
                            Some("system is inconsistent or has no unique solution".to_string());
                    }
                }
            }

            //if there is an error then show it
            if let Some(err_msg) = &self.error {
                ui.colored_label(egui::Color32::LIGHT_RED, err_msg);
            }

            if self.error.is_none() {
                if let Some(solutions) = &self.solution {
                    ui.add_space(10.0);
                    ui.heading("Solution:");

                    let labels = ["x", "y", "z", "w", "a", "b", "c", "d", "e", "f"];

                    for (i, val) in solutions.iter().enumerate() {
                        let label = labels
                            .get(i)
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| format!("x{}", i));

                        let formatted_val = if val.fract() == 0.0 {
                            format!("{}", val) //shows "7" instead of "7.0000"
                        } else {
                            format!("{:.4}", val) //shows "7.1234"
                        };

                        ui.label(format!("{} = {}", label, formatted_val));
                    }
                }
            }
        });
    }
}

fn solve(matrix: Vec<Vec<f64>>, results: Vec<f64>) -> Option<Vec<f64>> {
    //variable to not repeat code
    let n = matrix.len();
    //make augmented matrix
    let mut augmented = vec![vec![0.0; n + 1]; n];

    for i in 0..n {
        //copy the square matrix
        for j in 0..n {
            augmented[i][j] = matrix[i][j];
        }
        //at the end of each row add the result
        augmented[i][n] = results[i];
    }

    for collumn in 0..n {
        let mut max_row = collumn;
        let mut max_val = augmented[collumn][collumn].abs();

        //find the row with the largest value in the current column
        for i in (collumn + 1)..n {
            if augmented[i][collumn].abs() > max_val {
                max_val = augmented[i][collumn].abs();
                max_row = i;
            }
        }

        //swap rows if a better pivot was found
        if max_row != collumn {
            augmented.swap(collumn, max_row);
        }

        if augmented[collumn][collumn].abs() < 1e-10 {
            return None; //can't proceed in this case
        }

        for row in collumn + 1..n {
            //calculate the factor
            let factor = augmented[row][collumn] / augmented[collumn][collumn];

            for j in collumn..(n + 1) {
                augmented[row][j] -= factor * augmented[collumn][j];
            }
        }
    }

    let mut final_results = vec![0.0; n];

    for collumn in (0..n).rev() {
        let mut sum = augmented[collumn][n];
        for i in (collumn + 1)..n {
            sum -= augmented[collumn][i] * final_results[i]
        }
        let pivot = augmented[collumn][collumn];
        if pivot.abs() < 1e-10 {
            //if the number is too close to 0 assume there is no correct solution
            return None;
        }

        final_results[collumn] = sum / pivot;
    }

    Some(final_results)
}
