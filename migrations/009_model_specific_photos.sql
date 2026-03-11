-- Update all cars with model-specific Unsplash photos
-- Each car now shows actual photos matching its make/model

-- ============================================================
-- ORIGINAL 8 CARS (from 002_additions_and_seed.sql)
-- ============================================================

-- Toyota Camry Silver (Lekki)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1621007947382-bb3c3994e3fb?w=800&q=80',
    'https://images.unsplash.com/photo-1590362891991-f776e747a588?w=800&q=80',
    'https://images.unsplash.com/photo-1550355291-bbee04a92027?w=800&q=80'
] WHERE id = '11111111-aaaa-1111-aaaa-111111111111';

-- Honda Accord Black (VI)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1571095839087-5f237247ee28?w=800&q=80',
    'https://images.unsplash.com/photo-1577112319788-377a2131e05b?w=800&q=80',
    'https://images.unsplash.com/photo-1609676671207-d021525a635d?w=800&q=80'
] WHERE id = '22222222-bbbb-2222-bbbb-222222222222';

-- Mercedes-Benz GLE 350 White (Ikoyi)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1618843479313-40f8afb4b4d8?w=800&q=80',
    'https://images.unsplash.com/photo-1589667679645-cadf2f3139f2?w=800&q=80',
    'https://images.unsplash.com/photo-1542230387-bfc77d26903e?w=800&q=80'
] WHERE id = '33333333-cccc-3333-cccc-333333333333';

-- Toyota Highlander Blue (Ajah)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1594502184342-2e12f877aa73?w=800&q=80',
    'https://images.unsplash.com/photo-1533473359331-0135ef1b58bf?w=800&q=80',
    'https://images.unsplash.com/photo-1632137924251-fcea5ff46035?w=800&q=80'
] WHERE id = '44444444-dddd-4444-dddd-444444444444';

-- Lexus RX 350 Grey (Ikeja)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1568074532337-80968739967c?w=800&q=80',
    'https://images.unsplash.com/photo-1577496549804-8b05f1f67338?w=800&q=80',
    'https://images.unsplash.com/photo-1565157766821-580c4a1758cf?w=800&q=80'
] WHERE id = '55555555-eeee-5555-eeee-555555555555';

-- Toyota Corolla White (Surulere)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1590362891991-f776e747a588?w=800&q=80',
    'https://images.unsplash.com/photo-1621007947382-bb3c3994e3fb?w=800&q=80',
    'https://images.unsplash.com/photo-1550355291-bbee04a92027?w=800&q=80'
] WHERE id = '66666666-ffff-6666-ffff-666666666666';

-- Range Rover Sport Black (Banana Island)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1555404610-4f6162df064d?w=800&q=80',
    'https://images.unsplash.com/photo-1563458563737-e60b1f1b345f?w=800&q=80',
    'https://images.unsplash.com/photo-1652741938599-42b4aa7f1259?w=800&q=80'
] WHERE id = '77777777-aaaa-7777-aaaa-777777777777';

-- Honda CR-V Red (Yaba)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1748581699671-465009d8e2dd?w=800&q=80',
    'https://images.unsplash.com/photo-1609521263047-f8f205293f24?w=800&q=80',
    'https://images.unsplash.com/photo-1646029642262-022158ff5794?w=800&q=80'
] WHERE id = '88888888-bbbb-8888-bbbb-888888888888';

-- ============================================================
-- 007 MASSIVE MOCK DATA CARS (35 cars)
-- ============================================================

-- ABUJA --

-- Toyota Land Cruiser White (Wuse 2, Abuja)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1650530579355-7ad9d4766043?w=800&q=80',
    'https://images.unsplash.com/photo-1576676825635-472f74a821ec?w=800&q=80',
    'https://images.unsplash.com/photo-1572629166063-011a332eafed?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000001';

-- BMW X5 Black (Maitama, Abuja)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1635990215241-4d2805d729bb?w=800&q=80',
    'https://images.unsplash.com/photo-1555215695-3004980ad54e?w=800&q=80',
    'https://images.unsplash.com/photo-1628706268635-4dfcf2ac445a?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000002';

-- Mercedes-Benz C300 Silver (Garki, Abuja)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1597274394071-b7362c4a54ec?w=800&q=80',
    'https://images.unsplash.com/photo-1589667679645-cadf2f3139f2?w=800&q=80',
    'https://images.unsplash.com/photo-1592309905620-e5b59f6dcb98?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000003';

-- Toyota Corolla Grey (Kubwa, Abuja)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1590362891991-f776e747a588?w=800&q=80',
    'https://images.unsplash.com/photo-1621007947382-bb3c3994e3fb?w=800&q=80',
    'https://images.unsplash.com/photo-1550355291-bbee04a92027?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000004';

-- Lexus LX 570 Black (Asokoro, Abuja)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1650530579355-7ad9d4766043?w=800&q=80',
    'https://images.unsplash.com/photo-1568074532337-80968739967c?w=800&q=80',
    'https://images.unsplash.com/photo-1577496549804-8b05f1f67338?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000005';

-- PORT HARCOURT --

-- Toyota RAV4 Blue (GRA Phase 2, PH)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1632137924251-fcea5ff46035?w=800&q=80',
    'https://images.unsplash.com/photo-1594502184342-2e12f877aa73?w=800&q=80',
    'https://images.unsplash.com/photo-1533473359331-0135ef1b58bf?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000006';

-- Honda Pilot White (Trans Amadi, PH)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1609521263047-f8f205293f24?w=800&q=80',
    'https://images.unsplash.com/photo-1748581699671-465009d8e2dd?w=800&q=80',
    'https://images.unsplash.com/photo-1609676671207-d021525a635d?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000007';

-- Mercedes-Benz GLC 300 Black (Old GRA, PH)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1618843479313-40f8afb4b4d8?w=800&q=80',
    'https://images.unsplash.com/photo-1597274394071-b7362c4a54ec?w=800&q=80',
    'https://images.unsplash.com/photo-1542230387-bfc77d26903e?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000008';

-- ENUGU --

-- Toyota Hilux White (Independence Layout, Enugu)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1557863618-9643198cb07b?w=800&q=80',
    'https://images.unsplash.com/photo-1594502184342-2e12f877aa73?w=800&q=80',
    'https://images.unsplash.com/photo-1533473359331-0135ef1b58bf?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000009';

-- Kia Sportage Red (New Haven, Enugu)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1646029642262-022158ff5794?w=800&q=80',
    'https://images.unsplash.com/photo-1597220542065-dbd32fb169f9?w=800&q=80',
    'https://images.unsplash.com/photo-1609521263047-f8f205293f24?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000010';

-- IBADAN --

-- Toyota Camry Black (Bodija, Ibadan)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1621007947382-bb3c3994e3fb?w=800&q=80',
    'https://images.unsplash.com/photo-1590362891991-f776e747a588?w=800&q=80',
    'https://images.unsplash.com/photo-1550355291-bbee04a92027?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000011';

-- Hyundai Tucson Grey (Ring Road, Ibadan)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1646029642262-022158ff5794?w=800&q=80',
    'https://images.unsplash.com/photo-1609521263047-f8f205293f24?w=800&q=80',
    'https://images.unsplash.com/photo-1632137924251-fcea5ff46035?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000012';

-- Lexus ES 350 White (Jericho, Ibadan)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1568074532337-80968739967c?w=800&q=80',
    'https://images.unsplash.com/photo-1565157766821-580c4a1758cf?w=800&q=80',
    'https://images.unsplash.com/photo-1577496549804-8b05f1f67338?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000013';

-- BENIN CITY --

-- Toyota Venza Silver (GRA, Benin City)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1594502184342-2e12f877aa73?w=800&q=80',
    'https://images.unsplash.com/photo-1621007947382-bb3c3994e3fb?w=800&q=80',
    'https://images.unsplash.com/photo-1533473359331-0135ef1b58bf?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000014';

-- Ford Explorer Blue (Sapele Road, Benin City)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1533473359331-0135ef1b58bf?w=800&q=80',
    'https://images.unsplash.com/photo-1609521263047-f8f205293f24?w=800&q=80',
    'https://images.unsplash.com/photo-1632137924251-fcea5ff46035?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000015';

-- CALABAR --

-- Nissan Pathfinder Black (Marian Road, Calabar)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1609521263047-f8f205293f24?w=800&q=80',
    'https://images.unsplash.com/photo-1533473359331-0135ef1b58bf?w=800&q=80',
    'https://images.unsplash.com/photo-1594502184342-2e12f877aa73?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000016';

-- KANO --

-- Toyota Prado White (Nassarawa GRA, Kano)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1576676825635-472f74a821ec?w=800&q=80',
    'https://images.unsplash.com/photo-1650530579355-7ad9d4766043?w=800&q=80',
    'https://images.unsplash.com/photo-1572629166063-011a332eafed?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000017';

-- Honda Accord Silver (Sabon Gari, Kano)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1571095839087-5f237247ee28?w=800&q=80',
    'https://images.unsplash.com/photo-1577112319788-377a2131e05b?w=800&q=80',
    'https://images.unsplash.com/photo-1609676671207-d021525a635d?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000018';

-- KADUNA --

-- Audi Q5 Grey (Barnawa, Kaduna)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1605494540522-88bad6003ea2?w=800&q=80',
    'https://images.unsplash.com/photo-1564544422008-d0fb0c42c010?w=800&q=80',
    'https://images.unsplash.com/photo-1555215695-3004980ad54e?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000019';

-- MORE LAGOS --

-- Porsche Cayenne White (VI, Lagos)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1654159866298-e3c8ee93e43b?w=800&q=80',
    'https://images.unsplash.com/photo-1643055359228-9c33c2a2b9b3?w=800&q=80',
    'https://images.unsplash.com/photo-1606016159991-dfe4f2746ad5?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000020';

-- Nissan Altima Blue (Festac Town, Lagos)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1590362891991-f776e747a588?w=800&q=80',
    'https://images.unsplash.com/photo-1621007947382-bb3c3994e3fb?w=800&q=80',
    'https://images.unsplash.com/photo-1550355291-bbee04a92027?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000021';

-- BMW 5 Series Black (Ikoyi, Lagos)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1555215695-3004980ad54e?w=800&q=80',
    'https://images.unsplash.com/photo-1628706268635-4dfcf2ac445a?w=800&q=80',
    'https://images.unsplash.com/photo-1635990215241-4d2805d729bb?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000022';

-- Toyota Sienna Grey (Maryland, Lagos)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1594502184342-2e12f877aa73?w=800&q=80',
    'https://images.unsplash.com/photo-1533473359331-0135ef1b58bf?w=800&q=80',
    'https://images.unsplash.com/photo-1609521263047-f8f205293f24?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000023';

-- Mazda CX-5 Red (Gbagada, Lagos)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1646029642262-022158ff5794?w=800&q=80',
    'https://images.unsplash.com/photo-1597220542065-dbd32fb169f9?w=800&q=80',
    'https://images.unsplash.com/photo-1609521263047-f8f205293f24?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000024';

-- OWERRI --

-- Toyota Highlander Black (New Owerri)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1594502184342-2e12f877aa73?w=800&q=80',
    'https://images.unsplash.com/photo-1572629166063-011a332eafed?w=800&q=80',
    'https://images.unsplash.com/photo-1533473359331-0135ef1b58bf?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000025';

-- WARRI --

-- Toyota 4Runner Green (Effurun, Warri)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1557863618-9643198cb07b?w=800&q=80',
    'https://images.unsplash.com/photo-1594502184342-2e12f877aa73?w=800&q=80',
    'https://images.unsplash.com/photo-1632137924251-fcea5ff46035?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000026';

-- ABEOKUTA --

-- Hyundai Elantra White (Oke-Mosan, Abeokuta)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1590362891991-f776e747a588?w=800&q=80',
    'https://images.unsplash.com/photo-1621007947382-bb3c3994e3fb?w=800&q=80',
    'https://images.unsplash.com/photo-1550355291-bbee04a92027?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000027';

-- ASABA --

-- Mitsubishi Pajero Silver (Cable Point, Asaba)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1594502184342-2e12f877aa73?w=800&q=80',
    'https://images.unsplash.com/photo-1572629166063-011a332eafed?w=800&q=80',
    'https://images.unsplash.com/photo-1557863618-9643198cb07b?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000028';

-- UMUAHIA --

-- Honda CR-V Blue (World Bank, Umuahia)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1748581699671-465009d8e2dd?w=800&q=80',
    'https://images.unsplash.com/photo-1609521263047-f8f205293f24?w=800&q=80',
    'https://images.unsplash.com/photo-1609676671207-d021525a635d?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000029';

-- JOS --

-- Jeep Grand Cherokee Black (Rayfield, Jos)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1578546556117-a6b99653c632?w=800&q=80',
    'https://images.unsplash.com/photo-1609608934434-0f67a376da2a?w=800&q=80',
    'https://images.unsplash.com/photo-1742070850249-a19315d1291c?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000030';

-- ILORIN --

-- Toyota Camry Red (GRA, Ilorin)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1621007947382-bb3c3994e3fb?w=800&q=80',
    'https://images.unsplash.com/photo-1590362891991-f776e747a588?w=800&q=80',
    'https://images.unsplash.com/photo-1550355291-bbee04a92027?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000031';

-- AKURE --

-- Volkswagen Tiguan White (Alagbaka, Akure)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1605494540522-88bad6003ea2?w=800&q=80',
    'https://images.unsplash.com/photo-1564544422008-d0fb0c42c010?w=800&q=80',
    'https://images.unsplash.com/photo-1609521263047-f8f205293f24?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000032';

-- UYO --

-- Lexus RX 350 Grey (Ewet Housing, Uyo)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1568074532337-80968739967c?w=800&q=80',
    'https://images.unsplash.com/photo-1577496549804-8b05f1f67338?w=800&q=80',
    'https://images.unsplash.com/photo-1565157766821-580c4a1758cf?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000033';

-- LEKKI / AJAH PREMIUM --

-- Range Rover Velar Grey (Lekki Phase 1, Lagos)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1555404610-4f6162df064d?w=800&q=80',
    'https://images.unsplash.com/photo-1563458563737-e60b1f1b345f?w=800&q=80',
    'https://images.unsplash.com/photo-1606016159991-dfe4f2746ad5?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000034';

-- Mercedes-Benz S-Class Black (Banana Island, Lagos)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1514316454349-750a7fd3da3a?w=800&q=80',
    'https://images.unsplash.com/photo-1559167628-9394a8576f33?w=800&q=80',
    'https://images.unsplash.com/photo-1542230387-bfc77d26903e?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000035';

-- ELECTRIC VEHICLES --

-- Tesla Model 3 White (Lekki, Lagos)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1560958089-b8a1929cea89?w=800&q=80',
    'https://images.unsplash.com/photo-1561580125-028ee3bd62eb?w=800&q=80',
    'https://images.unsplash.com/photo-1610470832703-95d40c3fad55?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000036';

-- BMW iX Blue (VI, Lagos)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1555215695-3004980ad54e?w=800&q=80',
    'https://images.unsplash.com/photo-1635990215241-4d2805d729bb?w=800&q=80',
    'https://images.unsplash.com/photo-1628706268635-4dfcf2ac445a?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000037';

-- BUDGET OPTIONS --

-- Suzuki Swift Red (Oshodi, Lagos)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1590362891991-f776e747a588?w=800&q=80',
    'https://images.unsplash.com/photo-1550355291-bbee04a92027?w=800&q=80',
    'https://images.unsplash.com/photo-1621007947382-bb3c3994e3fb?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000038';

-- Kia Rio Silver (Nyanya, Abuja)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1590362891991-f776e747a588?w=800&q=80',
    'https://images.unsplash.com/photo-1550355291-bbee04a92027?w=800&q=80',
    'https://images.unsplash.com/photo-1609521263047-f8f205293f24?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000039';

-- 2-SEATER SPORTS --

-- Mercedes-Benz AMG GT Red (Lekki Phase 1, Lagos)
UPDATE cars SET photos = ARRAY[
    'https://images.unsplash.com/photo-1553440569-bcc63803a83d?w=800&q=80',
    'https://images.unsplash.com/photo-1695427984584-5f50880f46f4?w=800&q=80',
    'https://images.unsplash.com/photo-1741014150642-a40a93af4fad?w=800&q=80'
] WHERE id = 'a0000001-0001-0001-0001-000000000040';
