-- Add multiple photos per car for image carousel
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1621007947382-bb3c3994e3fb?w=800&q=80',
    'https://images.unsplash.com/photo-1621007947382-bb3c3994e3fb?w=800&q=60',
    'https://images.unsplash.com/photo-1550355291-bbee04a92027?w=800&q=80'
] WHERE id = '11111111-aaaa-1111-aaaa-111111111111';

UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1619682817481-e994891cd1f5?w=800&q=80',
    'https://images.unsplash.com/photo-1631295868223-63265b40d9e4?w=800&q=80',
    'https://images.unsplash.com/photo-1609521263047-f8f205293f24?w=800&q=80'
] WHERE id = '22222222-bbbb-2222-bbbb-222222222222';

UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1618843479313-40f8afb4b4d8?w=800&q=80',
    'https://images.unsplash.com/photo-1617531653332-bd46c24f2068?w=800&q=80',
    'https://images.unsplash.com/photo-1606016159991-dfe4f2746ad5?w=800&q=80'
] WHERE id = '33333333-cccc-3333-cccc-333333333333';

UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1533473359331-0135ef1b58bf?w=800&q=80',
    'https://images.unsplash.com/photo-1550355291-bbee04a92027?w=800&q=80',
    'https://images.unsplash.com/photo-1621007947382-bb3c3994e3fb?w=800&q=80'
] WHERE id = '44444444-dddd-4444-dddd-444444444444';

UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1606611013016-969c19ba27bb?w=800&q=80',
    'https://images.unsplash.com/photo-1617531653332-bd46c24f2068?w=800&q=80',
    'https://images.unsplash.com/photo-1631295868223-63265b40d9e4?w=800&q=80'
] WHERE id = '55555555-eeee-5555-eeee-555555555555';

UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1590362891991-f776e747a588?w=800&q=80',
    'https://images.unsplash.com/photo-1550355291-bbee04a92027?w=800&q=80',
    'https://images.unsplash.com/photo-1533473359331-0135ef1b58bf?w=800&q=80'
] WHERE id = '66666666-ffff-6666-ffff-666666666666';

UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1606016159991-dfe4f2746ad5?w=800&q=80',
    'https://images.unsplash.com/photo-1617531653332-bd46c24f2068?w=800&q=80',
    'https://images.unsplash.com/photo-1618843479313-40f8afb4b4d8?w=800&q=80'
] WHERE id = '77777777-aaaa-7777-aaaa-777777777777';

UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1609521263047-f8f205293f24?w=800&q=80',
    'https://images.unsplash.com/photo-1619682817481-e994891cd1f5?w=800&q=80',
    'https://images.unsplash.com/photo-1550355291-bbee04a92027?w=800&q=80'
] WHERE id = '88888888-bbbb-8888-bbbb-888888888888';
