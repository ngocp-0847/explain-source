module.exports = {
  apps: [
    {
      name: 'qa-chatbot-frontend',
      script: 'npm',
      args: 'start',
      cwd: '/home/deploy/source/explain-source',
      instances: 1,
      autorestart: true,
      watch: false,
      max_memory_restart: '1G',
      env: {
        NODE_ENV: 'production',
        PORT: 3010
      },
      env_production: {
        NODE_ENV: 'production',
        PORT: 3010
      },
      log_file: '/home/deploy/source/explain-source/logs/frontend.log',
      out_file: '/home/deploy/source/explain-source/logs/frontend-out.log',
      error_file: '/home/deploy/source/explain-source/logs/frontend-error.log',
      log_date_format: 'YYYY-MM-DD HH:mm:ss Z',
      merge_logs: true,
      time: true
    },
    {
      name: 'qa-chatbot-backend',
      script: '/home/deploy/source/explain-source/rust-backend/target/release/qa-chatbot-backend',
      cwd: '/home/deploy/source/explain-source/rust-backend',
      instances: 1,
      autorestart: true,
      watch: false,
      max_memory_restart: '1G',
      env: {
        RUST_LOG: 'info',
        RUST_BACKTRACE: '1'
      },
      log_file: '/home/deploy/source/explain-source/logs/backend.log',
      out_file: '/home/deploy/source/explain-source/logs/backend-out.log',
      error_file: '/home/deploy/source/explain-source/logs/backend-error.log',
      log_date_format: 'YYYY-MM-DD HH:mm:ss Z',
      merge_logs: true,
      time: true
    }
  ]
};