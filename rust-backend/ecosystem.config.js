module.exports = {
  apps: [
    {
      name: 'qa-chatbot-frontend',
      script: 'npm',
      args: 'run dev',
      cwd: './',
      watch: false,
      env: {
        NODE_ENV: 'development',
        PORT: 3010
      },
      error_file: './logs/frontend-error.log',
      out_file: './logs/frontend-out.log',
      log_date_format: 'YYYY-MM-DD HH:mm:ss Z'
    },
    {
      name: 'qa-chatbot-backend',
      script: 'cargo',
      args: 'run',
      cwd: './rust-backend',
      watch: false,
      interpreter: 'none',
      env: {
        RUST_LOG: 'info',
        RUST_BACKTRACE: '1'
      },
      error_file: './logs/backend-error.log',
      out_file: './logs/backend-out.log',
      log_date_format: 'YYYY-MM-DD HH:mm:ss Z'
    }
  ]
};
