#!/usr/bin/env node

// Simple test server to verify hc deal command
console.log('=== Environment Variables Test Server ===\n');

const envVars = [
  'USERNAME',
  'PASSWORD',
  'API_KEY',
  'DATABASE_URL',
  'NODE_ENV',
  // Lower case versions
  'username',
  'password',
  'api_key',
  'database_url',
];

console.log('Checking environment variables:\n');

envVars.forEach(key => {
  const value = process.env[key];
  if (value) {
    // Mask sensitive values (show only first 3 chars)
    const masked = value.length > 3
      ? value.substring(0, 3) + '*'.repeat(Math.min(value.length - 3, 10))
      : '*'.repeat(value.length);
    console.log(`✓ ${key}: ${masked}`);
  } else {
    console.log(`✗ ${key}: (not set)`);
  }
});

console.log('\n=== All Environment Variables ===\n');
Object.keys(process.env)
  .filter(key => !key.startsWith('_') && !key.includes('PATH'))
  .sort()
  .forEach(key => {
    const value = process.env[key];
    const masked = value.length > 10
      ? value.substring(0, 10) + '...'
      : value;
    console.log(`${key}=${masked}`);
  });

console.log('\n✅ Server test completed successfully!');
