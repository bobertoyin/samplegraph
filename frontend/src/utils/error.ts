export default class Error {
    private _status?: number;
    private _statusText?: String;
    private _message?: String;

    public async setFromResponse(response: Response): Promise<void> {
        this._status = response.status;
        this._statusText = response.statusText;
        this._message = await response.text();
    }

    public get status(): number | undefined {
        return this._status;
    }

    public get statusText(): String | undefined {
        return this._statusText;
    }

    public get message(): String | undefined {
        return this._message;
    }

    public isSet(): boolean {
        return (
            this.status !== undefined || this.statusText !== undefined || this.message !== undefined
        );
    }

    public reset(): void {
        this._status = undefined;
        this._statusText = undefined;
        this._message = undefined;
    }
}
