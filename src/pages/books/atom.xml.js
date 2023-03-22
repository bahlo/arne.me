import rss from '@astrojs/rss';
import { getCollection } from 'astro:content';
import { SITE_TITLE, SITE_DESCRIPTION } from '../../consts';

export async function get(context) {
	const books = await getCollection('book');
	return rss({
		title: `${SITE_TITLE} — Books`,
		description: SITE_DESCRIPTION,
		site: context.site,
		items: books.map(book => ({
			...book.data,
			link: `/book/${book.slug}/`,
		})),
	});
}
