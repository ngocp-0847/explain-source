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
      watch: true,
      watch_delay: 2000,
      ignore_watch: [
        'target',
        '.git',
        'logs',
        '*.log',
        'Cargo.lock'
      ],
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
