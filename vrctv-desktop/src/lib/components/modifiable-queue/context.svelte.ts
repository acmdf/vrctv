import { createContext } from 'svelte';

type Getter<T> = () => T;

export type QueueCollapseStateProps = {
	/**
	 * A getter function that returns the current open state of the sidebar.
	 * We use a getter function here to support `bind:open` on the `Sidebar.Provider`
	 * component.
	 */
	open: Getter<boolean>;

	/**
	 * A function that sets the open state of the sidebar. To support `bind:open`, we need
	 * a source of truth for changing the open state to ensure it will be synced throughout
	 * the sub-components and any `bind:` references.
	 */
	setOpen: (open: boolean) => void;
};

export class QueueCollapseState {
	readonly props: QueueCollapseStateProps;
	open = $derived.by(() => this.props.open());
	setOpen: QueueCollapseStateProps["setOpen"];
	state = $derived.by(() => (this.open ? "expanded" : "collapsed"));

	constructor(props: QueueCollapseStateProps) {
		this.setOpen = props.setOpen;
		this.props = props;
	}

	toggle = () => {
		return this.setOpen(!this.open);
	};
}

export const [getCollapseContext, setCollapseContext] = createContext<QueueCollapseState>();