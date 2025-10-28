const path = require('path');
const os = require('os');
const PROJECT_ROOT = __dirname;

// Tìm cargo trong PATH
const findCargo = () => {
  const cargoPaths = [
    path.join(os.homedir(), '.cargo', 'bin', 'cargo'),
    '/usr/local/bin/cargo',
    '/usr/bin/cargo'
  ];
  
  for (const cargoPath of cargoPaths) {
    try {
      require('fs').accessSync(cargoPath, require('fs').constants.F_OK);
      return cargoPath;
    } catch (e) {
      continue;
    }
  }
  return 'cargo'; // Fallback to PATH
};

const cargoPath = findCargo();

module.exports = {
  apps: [
    {
      name: 'qa-chatbot-frontend',
      script: 'npm',
      args: 'run dev',
      cwd: PROJECT_ROOT,
      exec_mode: 'fork',
      instances: 1,
      autorestart: true,
      watch: false,
      max_memory_restart: '1G',
      interpreter: 'none',
      env: {
        NODE_ENV: 'development',
        PORT: 3010
      },
      log_file: path.join(PROJECT_ROOT, 'logs', 'frontend.log'),
      out_file: path.join(PROJECT_ROOT, 'logs', 'frontend-out.log'),
      error_file: path.join(PROJECT_ROOT, 'logs', 'frontend-error.log'),
      log_date_format: 'YYYY-MM-DD HH:mm:ss Z',
      merge_logs: true,
      time: true
    },
    {
      name: 'qa-chatbot-backend',
      script: cargoPath,
      args: 'run',
      cwd: path.join(PROJECT_ROOT, 'rust-backend'),
      exec_mode: 'fork',
      interpreter: 'none',
      instances: 1,
      autorestart: true,
      watch: false,
      max_memory_restart: '1G',
      env: {
        RUST_LOG: 'info',
        RUST_BACKTRACE: '1',
        PATH: `${process.env.HOME}/.cargo/bin:${process.env.PATH}`
      },
      log_file: path.join(PROJECT_ROOT, 'logs', 'backend.log'),
      out_file: path.join(PROJECT_ROOT, 'logs', 'backend-out.log'),
      error_file: path.join(PROJECT_ROOT, 'logs', 'backend-error.log'),
      log_date_format: 'YYYY-MM-DD HH:mm:ss Z',
      merge_logs: true,
      time: true
    }
  ]
};