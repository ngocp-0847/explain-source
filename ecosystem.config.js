module.exports = {
  apps: [
    {
      name: 'qa-chatbot-frontend',
      script: 'npm',
      args: 'run dev',
      cwd: '/Users/ngocp/Documents/projects/explain-source',
      instances: 1,
      autorestart: true,
      watch: false,
      max_memory_restart: '1G',
      env: {
        NODE_ENV: 'development',
        PORT: 3010
      },
      log_file: '/Users/ngocp/Documents/projects/explain-source/logs/frontend.log',
      out_file: '/Users/ngocp/Documents/projects/explain-source/logs/frontend-out.log',
      error_file: '/Users/ngocp/Documents/projects/explain-source/logs/frontend-error.log',
      log_date_format: 'YYYY-MM-DD HH:mm:ss Z',
      merge_logs: true,
      time: true
    },
    {
      name: 'qa-chatbot-backend',
      script: '/Users/ngocp/.cargo/bin/cargo',
      args: 'run',
      cwd: '/Users/ngocp/Documents/projects/explain-source/rust-backend',
      interpreter: 'none',
      instances: 1,
      autorestart: true,
      watch: false,
      max_memory_restart: '1G',
      env: {
        RUST_LOG: 'info',
        RUST_BACKTRACE: '1'
      },
      log_file: '/Users/ngocp/Documents/projects/explain-source/logs/backend.log',
      out_file: '/Users/ngocp/Documents/projects/explain-source/logs/backend-out.log',
      error_file: '/Users/ngocp/Documents/projects/explain-source/logs/backend-error.log',
      log_date_format: 'YYYY-MM-DD HH:mm:ss Z',
      merge_logs: true,
      time: true
    }
  ]
};