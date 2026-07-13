export type ApiError = {
	type: 'api_error';
	message: string;
	status: number;
};

export const createApiError = (message: string, status: number): ApiError => {
	return { type: 'api_error', message, status };
};

export const apiRequest = async (
	path: string,
	method: string,
	body?: unknown
): Promise<unknown> => {
	let response: Response;

	try {
		response = await fetch(path, {
			method,
			headers: body === undefined ? undefined : { 'Content-Type': 'application/json' },
			body: body === undefined ? undefined : JSON.stringify(body)
		});
	} catch {
		throw createApiError('Could not reach the server. Please try again.', 0);
	}

	if (!response.ok) {
		let message = `Request failed (${response.status})`;
		try {
			const data = await response.json();
			if (data && typeof data.error === 'string') {
				message = data.error;
			}
		} catch {
			// Response has no JSON body
		}
		throw createApiError(message, response.status);
	}

	if (response.status === 204) {
		return null;
	}

	try {
		return await response.json();
	} catch {
		return null;
	}
};
