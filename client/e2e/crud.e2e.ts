import { expect, test } from '@playwright/test';

test('cardio session create, read, update, and delete in order', async ({ page }) => {
	const date = '2099-06-15';
	const createdName = 'Treadmill CRUD';
	const updatedName = 'Bike CRUD';
	const createdDuration = '25';
	const updatedDuration = '40';

	await page.goto('/');
	await page.locator('input[type="date"]').fill(date);
	await expect(page.getByRole('button', { name: 'Edit day' })).toBeVisible();

	// Create
	await page.getByRole('button', { name: 'Edit day' }).click();
	await page.getByRole('button', { name: 'Add cardio' }).click();
	await page.getByLabel('Exercise name').fill(createdName);
	await page.getByLabel('Duration (minutes)').fill(createdDuration);
	await page.getByRole('button', { name: 'Save', exact: true }).click();
	await expect(page.getByRole('button', { name: createdName })).toBeVisible();

	// Read
	await page.getByRole('button', { name: 'View day' }).click();
	await expect(page.getByText(createdName)).toBeVisible();
	await expect(page.getByText(`${createdDuration} min`)).toBeVisible();

	// Update
	await page.getByRole('button', { name: 'Edit day' }).click();
	await page.getByRole('button', { name: createdName }).click();
	await page.getByLabel('Exercise name').fill(updatedName);
	await page.getByLabel('Duration (minutes)').fill(updatedDuration);
	await page.getByRole('button', { name: 'Save', exact: true }).click();
	await page.getByRole('button', { name: 'View day' }).click();
	await expect(page.getByText(updatedName)).toBeVisible();
	await expect(page.getByText(`${updatedDuration} min`)).toBeVisible();
	await expect(page.getByText(createdName)).toHaveCount(0);

	// Delete
	await page.getByRole('button', { name: 'Edit day' }).click();
	await page.getByRole('button', { name: updatedName }).click();
	await page.getByRole('button', { name: 'Delete' }).click();
	await page.getByRole('button', { name: 'View day' }).click();
	await expect(page.getByText(updatedName)).toHaveCount(0);
	await expect(page.getByRole('heading', { name: 'Cardio' })).toHaveCount(0);
});
