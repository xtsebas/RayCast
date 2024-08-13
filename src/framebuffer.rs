use crate::bmp::write_bmp_file;
use rusttype::{Font, Scale, point};

pub struct Framebuffer {
    pub width: usize,  // Ancho del framebuffer
    pub height: usize, // Alto del framebuffer
    pub buffer: Vec<u32>,  // Buffer de píxeles
    background_color: u32, // Color de fondo del framebuffer
    current_color: u32,    // Color actual del framebuffer
}

impl Framebuffer {
    // Constructor
    pub fn new(width: usize, height: usize) -> Self {
        let buffer_size = width * height;
        let background_color = 0x000000; // Color de fondo predeterminado (negro)
        let buffer = vec![background_color; buffer_size]; // Crea un vector de tamaño buffer_size con el color de fondo
        Framebuffer {
            width, // Asigna el ancho proporcionado al campo width de la estructura
            height, // Asigna el alto proporcionado al campo height de la estructura
            buffer, // Asigna el vector de píxeles al campo buffer de la estructura
            background_color, // Asigna el color de fondo proporcionado al campo background_color de la estructura
            current_color: 0xFFFFFF, // Color actual predeterminado (blanco)
        }
    }

    // Función para limpiar el framebuffer
    pub fn clear(&mut self) {
        // Usa el método fill de Vec para llenar todo el buffer con el color de fondo
        self.buffer.fill(self.background_color);
    }

    // Función para establecer un punto en el framebuffer
    pub fn point(&mut self, x: usize, y: usize) {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            let index = (y as usize) * self.width + (x as usize); // Calcula el índice en el buffer para el punto (x, y)
            self.buffer[index] = self.current_color; // Establece el color actual en el punto correspondiente en el buffer
        }
    }

    // Función para obtener el color de un punto en el framebuffer
    pub fn get_point(&self, x: isize, y: isize) -> Option<u32> {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            let index = (y as usize) * self.width + (x as usize); // Calcula el índice en el buffer para el punto (x, y)
            Some(self.buffer[index]) // Devuelve el color del punto como Some(color)
        } else {
            None // Devuelve None si el punto está fuera de los límites del framebuffer
        }
    }

    // Establecer el color de fondo del framebuffer
    pub fn set_background_color(&mut self, color: u32) {
        self.background_color = color; // Actualiza el color de fondo del framebuffer
    }

    // Establecer el color actual del framebuffer
    pub fn set_current_color(&mut self, color: u32) {
        self.current_color = color; // Actualiza el color actual del framebuffer
    }

    pub fn drawtext(&mut self, text: &str, x: usize, y: usize, scale: Scale, color: u32) {
        let font_data = include_bytes!("../assets/Nasa.ttf") as &[u8];
        let font = Font::try_from_bytes(font_data).unwrap();
        let v_metrics = font.v_metrics(scale);
    
        let line_height = (v_metrics.ascent - v_metrics.descent) as usize;
    
        // Dividir el texto en líneas
        let lines: Vec<&str> = text.lines().collect();
        let mut current_y = y;
    
        for line in lines {
            let offset = point(x as f32, current_y as f32 + v_metrics.ascent);
    
            let glyphs: Vec<_> = font.layout(line, scale, offset).collect();
    
            for glyph in glyphs {
                if let Some(bounding_box) = glyph.pixel_bounding_box() {
                    glyph.draw(|gx, gy, gv| {
                        if gv > 0.5 {
                            let gx = gx as i32 + bounding_box.min.x;
                            let gy = gy as i32 + bounding_box.min.y;
    
                            if gx >= 0 && gx < self.width as i32 && gy >= 0 && gy < self.height as i32 {
                                let gx = gx as usize;
                                let gy = gy as usize;
    
                                self.buffer[gy * self.width + gx] = color;
                            }
                        }
                    });
                }
            }
    
            // Actualizar la posición vertical para la próxima línea
            current_y += line_height;
        }
    }

    // Función para devolver una referencia al buffer de píxeles
    pub fn render_buffer(&self, file_path: &str) {
        write_bmp_file(file_path, &self.buffer, self.width, self.height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_framebuffer() {
        let fb = Framebuffer::new(800, 600);
        assert_eq!(fb.width, 800);
        assert_eq!(fb.height, 600);
        assert_eq!(fb.buffer.len(), 800 * 600);
    }

    #[test]
    fn test_clear() {
        let mut fb = Framebuffer::new(800, 600);
        fb.set_background_color(0x00FF00); // Set background color to green
        fb.clear();

        for pixel in fb.buffer.iter() {
            assert_eq!(*pixel, 0x00FF00);
        }
    }

    #[test]
    fn test_point() {
        let mut fb = Framebuffer::new(800, 600);
        fb.set_current_color(0xFF0000); // Set current color to red
        fb.point(400, 300);

        let index = 300 * 800 + 400;
        assert_eq!(fb.buffer[index], 0xFF0000);

        // Test negative coordinates
        fb.point(-1, -1); // Should not panic
        fb.point(800, 600); // Out of bounds, should be ignored
    }

    #[test]
    fn test_get_point() {
        let mut fb = Framebuffer::new(800, 600);
        fb.set_current_color(0xFF0000); // Set current color to red
        fb.point(400, 300);

        assert_eq!(fb.get_point(400, 300), Some(0xFF0000));
        assert_eq!(fb.get_point(800, 600), None); // Out of bounds
        assert_eq!(fb.get_point(-1, -1), None); // Out of bounds (negative coordinates)
    }

    #[test]
    fn test_set_colors() {
        let mut fb = Framebuffer::new(800, 600);
        fb.set_background_color(0x00FF00); // Set background color to green
        fb.set_current_color(0xFF0000); // Set current color to red
        fb.clear();

        for pixel in fb.buffer.iter() {
            assert_eq!(*pixel, 0x00FF00);
        }

        fb.point(400, 300);
        let index = 300 * 800 + 400;
        assert_eq!(fb.buffer[index], 0xFF0000);
    }
}